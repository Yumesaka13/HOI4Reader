use super::{country_flag, index_mod, prepare_index, Country, CountryName, ScanResult};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    fs::{self, File},
    hash::{Hash, Hasher},
    io::{self, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
};
use tauri::{ipc::Channel, State};
use walkdir::WalkDir;

const SOURCE_REVISION: &str = concat!(
    include_str!("lib.rs"),
    include_str!("mod_layers.rs"),
    include_str!("reader_notes.rs"),
    include_str!("super_events.rs"),
    include_str!("index_cache.rs")
);

#[derive(Clone, Default)]
pub struct IndexStore(Arc<Mutex<IndexState>>);

#[derive(Default)]
struct IndexState {
    latest: u64,
    active: Option<ActiveIndex>,
}

#[derive(Clone)]
struct ActiveIndex {
    root: PathBuf,
    language: String,
    files: Option<CacheFiles>,
    graph: Arc<ScanResult>,
    refresh_secondary: bool,
}

#[derive(Clone)]
struct CacheFiles {
    prefix: PathBuf,
    source_stamp: u64,
}

#[derive(Serialize, Deserialize)]
struct PrimaryIndex {
    graph: Arc<ScanResult>,
    countries: Vec<Country>,
}

#[derive(Serialize)]
pub struct ScanResponse {
    #[serde(flatten)]
    graph: Arc<ScanResult>,
    countries: Vec<Country>,
    has_secondary_countries: bool,
    cache_warning: Option<String>,
}

#[derive(Serialize)]
pub struct SecondaryResponse {
    countries: Vec<Country>,
    cache_warning: Option<String>,
}

#[cfg(not(target_os = "android"))]
fn cache_directory() -> io::Result<PathBuf> {
    let executable = std::env::current_exe()?;
    #[cfg(target_os = "linux")]
    let executable = std::env::var_os("APPIMAGE")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute() && path.is_file())
        .unwrap_or(executable);
    #[cfg(target_os = "macos")]
    let executable = executable
        .ancestors()
        .find(|path| path.extension().is_some_and(|ext| ext == "app"))
        .map(Path::to_path_buf)
        .unwrap_or(executable);
    Ok(executable
        .parent()
        .ok_or(io::ErrorKind::NotFound)?
        .join("hoi4reader-index"))
}

fn stamp_directory(path: &Path, hash: &mut DefaultHasher) -> io::Result<()> {
    path.hash(hash);
    if !path.try_exists()? {
        return Ok(());
    }
    for entry in WalkDir::new(path).sort_by_file_name().follow_links(false) {
        let entry = entry?;
        if entry.file_type().is_file() || entry.file_type().is_symlink() {
            entry.path().hash(hash);
            let metadata = entry.metadata()?;
            metadata.len().hash(hash);
            metadata.modified()?.hash(hash);
        }
    }
    Ok(())
}

fn source_stamp(root: &Path) -> io::Result<u64> {
    let mut hash = DefaultHasher::new();
    SOURCE_REVISION.hash(&mut hash);
    for directory in [
        "common",
        "events",
        "localisation",
        "history/countries",
        "interface",
        "gfx/interface/goals",
        "gfx/event_pictures",
        "gfx/super_events",
        "gfx/flags",
        "flags",
        "music",
        "sound",
    ] {
        stamp_directory(&root.join(directory), &mut hash)?;
    }
    if let Some(parent) = root.parent() {
        let music_addon = parent.join("3350892196");
        for directory in ["music", "localisation", "sound"] {
            stamp_directory(&music_addon.join(directory), &mut hash)?;
        }
    }
    Ok(hash.finish())
}

impl CacheFiles {
    fn new(directory: PathBuf, root: &Path, language: &str, game: Option<&Path>) -> io::Result<Self> {
        fs::create_dir_all(&directory).map_err(|error| {
            io::Error::new(error.kind(), format!("{} ({error})", directory.display()))
        })?;
        let mut key = DefaultHasher::new();
        (root, language, game).hash(&mut key);
        Ok(Self {
            prefix: directory.join(format!("{:016x}", key.finish())),
            source_stamp: scan_stamp(root, language, game)?,
        })
    }

    fn read<T: DeserializeOwned>(&self, extension: &str) -> Option<T> {
        let mut reader = BufReader::new(File::open(self.prefix.with_extension(extension)).ok()?);
        let mut header = [0; 17];
        reader.read_exact(&mut header).ok()?;
        if header != format!("{:016x}\n", self.source_stamp).as_bytes() {
            return None;
        }
        serde_json::from_reader(reader).ok()
    }

    fn write<T: Serialize>(&self, extension: &str, data: &T) -> io::Result<()> {
        static NEXT_FILE: AtomicU64 = AtomicU64::new(0);
        let destination = self.prefix.with_extension(extension);
        let temporary = destination.with_extension(format!(
            "{extension}.{}.{}.tmp",
            std::process::id(),
            NEXT_FILE.fetch_add(1, Ordering::Relaxed),
        ));
        let file = File::options()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        let result = (|| {
            let mut writer = BufWriter::new(file);
            writeln!(writer, "{:016x}", self.source_stamp)?;
            serde_json::to_writer(&mut writer, data)?;
            writer.flush()?;
            drop(writer);
            fs::rename(&temporary, &destination)
        })();
        if result.is_err() {
            let _ = fs::remove_file(temporary);
        }
        result
    }
}

fn scan_stamp(root: &Path, language: &str, game: Option<&Path>) -> io::Result<u64> {
    let mut hash = DefaultHasher::new();
    for layer in super::mod_layers::localisation_roots(root) {
        source_stamp(&layer)?.hash(&mut hash);
        stamp_directory(&layer.join("descriptor.mod"), &mut hash)?;
    }
    if root
        .file_name()
        .is_some_and(|id| id == super::mod_layers::TFR_CHILD)
    {
        source_stamp(&root.with_file_name("3793754397"))?.hash(&mut hash);
    }
    if let Some(game) = game {
        stamp_directory(&game.join("interface/core.gfx"), &mut hash)?;
        for file in super::reader_notes::game_template_paths(game, language) {
            stamp_directory(&file, &mut hash)?;
        }
    }
    Ok(hash.finish())
}

fn has_major_countries(names: &[CountryName]) -> bool {
    names.iter().any(|country| country.major_order.is_some())
}

fn primary_names(names: &[CountryName]) -> impl Iterator<Item = &CountryName> {
    let has_majors = has_major_countries(names);
    names
        .iter()
        .filter(move |name| !has_majors || name.major_order.is_some())
}

fn country_catalog<'a>(
    root: &Path,
    graph: &'a ScanResult,
    names: impl Iterator<Item = &'a CountryName>,
) -> Vec<Country> {
    let mut countries = names
        .map(|name| {
            (
                name.tag.as_str(),
                Country {
                    tag: name.tag.clone(),
                    name: name.name.clone(),
                    major_order: name.major_order,
                    flag_image: None,
                    chapter_count: 0,
                    story_count: 0,
                    chapter_indices: Vec::new(),
                },
            )
        })
        .collect::<HashMap<_, _>>();
    for (index, chapter) in graph.chapters.iter().enumerate() {
        let inferred_owner = chapter
            .country_tags
            .is_empty()
            .then(|| chapter.id.split_once('_').map(|(tag, _)| tag))
            .flatten();
        for tag in chapter
            .country_tags
            .iter()
            .map(String::as_str)
            .chain(inferred_owner)
        {
            if let Some(country) = countries.get_mut(tag) {
                country.chapter_indices.push(index);
            }
        }
    }
    for item in graph.timeline.iter().filter(|item| !item.hidden) {
        if let Some(country) = countries.get_mut(item.country_tag.as_str()) {
            country.story_count += 1;
        }
    }
    let mut countries = countries.into_values().collect::<Vec<_>>();
    for country in &mut countries {
        country.chapter_count = country.chapter_indices.len();
        country.flag_image = country_flag(root, &country.tag);
    }
    countries.sort_by(|a, b| a.tag.cmp(&b.tag));
    countries
}

#[tauri::command]
pub async fn scan_mod(
    _app: tauri::AppHandle,
    path: String,
    language: String,
    game_path: Option<String>,
    force: bool,
    on_countries: Channel<Vec<CountryName>>,
    store: State<'_, IndexStore>,
) -> Result<ScanResponse, String> {
    let store = store.inner().clone();
    let request = {
        let mut state = store.0.lock().map_err(|error| error.to_string())?;
        state.latest += 1;
        state.latest
    };
    tauri::async_runtime::spawn_blocking(move || {
        let authored_root = PathBuf::from(path);
        let root = fs::canonicalize(&authored_root).map_err(|error| error.to_string())?;
        if let Some(parent) = super::mod_layers::parent_id(&root) {
            if super::mod_layers::parent(&root).is_none() {
                return Err(format!(
                    "Missing base mod ({parent}) in the same mod library."
                ));
            }
        }
        let game = game_path
            .as_deref()
            .filter(|path| !path.is_empty())
            .map(Path::new);
        #[cfg(target_os = "android")]
        let directory = {
            use tauri::Manager;
            _app.path().app_data_dir()
                .map(|path| path.join("hoi4reader-index"))
                .map_err(io::Error::other)
        };
        #[cfg(not(target_os = "android"))]
        let directory = cache_directory();
        let (mut files, mut cache_warning) = match directory.and_then(|directory| CacheFiles::new(directory, &root, &language, game)) {
            Ok(files) => (Some(files), None),
            Err(error) => (None, Some(error.to_string())),
        };
        let cached = files
            .as_ref()
            .filter(|_| !force)
            .and_then(|files| files.read::<PrimaryIndex>("story.json"));
        let primary = if let Some(cached) = cached {
            cached
        } else {
            let inputs = prepare_index(&authored_root, &language);
            let _ = on_countries.send(primary_names(&inputs.country_names).cloned().collect());
            let graph = Arc::new(index_mod(authored_root, &language, inputs, game));
            let countries = country_catalog(&root, &graph, primary_names(&graph.country_names));
            let primary = PrimaryIndex { graph, countries };
            if let Some(cache) = &files {
                let saved = scan_stamp(&root, &language, game).and_then(|current| {
                    if current != cache.source_stamp {
                        return Err(io::Error::other("Mod files changed during indexing"));
                    }
                    cache.write("story.json", &primary)
                });
                if let Err(error) = saved {
                    cache_warning = Some(error.to_string());
                    files = None;
                }
            }
            primary
        };
        let mut state = store.0.lock().map_err(|error| error.to_string())?;
        if state.latest == request {
            state.active = Some(ActiveIndex {
                root,
                language,
                files,
                graph: primary.graph.clone(),
                refresh_secondary: force,
            });
        }
        let has_secondary_countries = primary.countries.len() < primary.graph.country_names.len();
        Ok(ScanResponse {
            graph: primary.graph,
            countries: primary.countries,
            has_secondary_countries,
            cache_warning,
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn index_secondary_countries(
    path: String,
    language: String,
    store: State<'_, IndexStore>,
) -> Result<SecondaryResponse, String> {
    let store = store.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let root = fs::canonicalize(path).map_err(|error| error.to_string())?;
        let active = store
            .0
            .lock()
            .map_err(|error| error.to_string())?
            .active
            .clone()
            .filter(|active| active.root == root && active.language == language)
            .ok_or_else(|| "The selected mod's story index is not ready.".to_owned())?;
        let mut cache_warning = None;
        let cached = active
            .files
            .as_ref()
            .filter(|_| !active.refresh_secondary)
            .and_then(|files| files.read::<Vec<Country>>("secondary.json"));
        let countries = cached.unwrap_or_else(|| {
            let names = active
                .graph
                .country_names
                .iter()
                .filter(|name| name.major_order.is_none());
            let countries = country_catalog(&root, &active.graph, names);
            if let Some(files) = active.files {
                if let Err(error) = files.write("secondary.json", &countries) {
                    cache_warning = Some(error.to_string());
                }
            }
            countries
        });
        Ok(SecondaryResponse {
            countries,
            cache_warning,
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[cfg(test)]
#[test]
fn cached_indexes_are_reusable_until_the_mod_changes() -> io::Result<()> {
    let directory =
        std::env::temp_dir().join(format!("hoi4reader-index-test-{}", std::process::id()));
    fs::create_dir(&directory)?;
    let events = directory.join("events");
    fs::create_dir(&events)?;
    let script = events.join("story.txt");
    fs::write(&script, "original")?;
    let cache = CacheFiles {
        prefix: directory.join("index"),
        source_stamp: source_stamp(&directory)?,
    };
    cache.write("story.json", &vec!["JAP"])?;
    assert_eq!(
        cache.read::<Vec<String>>("story.json"),
        Some(vec!["JAP".into()])
    );
    assert!(cache.read::<Vec<String>>("secondary.json").is_none());
    fs::write(&script, "updated content")?;
    let updated = CacheFiles {
        prefix: cache.prefix,
        source_stamp: source_stamp(&directory)?,
    };
    assert!(updated.read::<Vec<String>>("story.json").is_none());
    updated.write("story.json", &vec!["PRC"])?;
    assert_eq!(
        updated.read::<Vec<String>>("story.json"),
        Some(vec!["PRC".into()])
    );
    fs::remove_file(updated.prefix.with_extension("story.json"))?;
    fs::remove_file(script)?;
    fs::remove_dir(events)?;
    fs::remove_dir(directory)
}
