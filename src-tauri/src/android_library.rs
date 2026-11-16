use std::{
    fs::{self, File},
    io::{self, Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::Mutex,
    time::{Duration, Instant},
};
use tauri::{ipc::Channel, AppHandle, Manager, Url};
use tauri_plugin_fs::{FilePath, FsExt, OpenOptions};

const MAX_ARCHIVE_BYTES: u64 = 32 * 1024 * 1024 * 1024;
const MAX_ENTRIES: usize = 500_000;
const GAME: &str = "game";
static IMPORT: Mutex<()> = Mutex::new(());

fn library(app: &AppHandle) -> tauri::Result<PathBuf> {
    Ok(app.path().app_data_dir()?.join("android-library"))
}

pub fn discover(app: &AppHandle) -> super::Discovery {
    let library = library(app).ok();
    let game = library.as_ref().map(|root| root.join(GAME));
    super::Discovery {
        game_path: game
            .filter(|path| path.is_dir())
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_default(),
        workshop_path: library
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_default(),
        mods: library.map(|path| super::scan_workshop(&path)).unwrap_or_default(),
    }
}

#[tauri::command]
pub async fn import_archive(app: AppHandle, uri: String) -> Result<super::Discovery, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _lock = IMPORT.try_lock().map_err(|error| error.to_string())?;
        let Ok(source) = uri.parse::<FilePath>();
        if !matches!(&source, FilePath::Url(url) if url.scheme() == "content") {
            return Err("Select a ZIP file through the Android file picker.".into());
        }
        let mut options = OpenOptions::new();
        options.read(true);
        let file = app.fs().open(source, options).map_err(|error| error.to_string())?;
        let root = library(&app).map_err(|error| error.to_string())?;
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        install_zip(file, &root).map_err(|error| error.to_string())?;
        Ok(discover(&app))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[derive(serde::Serialize)]
pub struct DownloadProgress {
    received: u64,
    total: Option<u64>,
    installing: bool,
}

#[tauri::command]
pub async fn download_archive(
    app: AppHandle,
    url: String,
    on_progress: Channel<DownloadProgress>,
) -> Result<super::Discovery, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let _lock = IMPORT.try_lock().map_err(|error| error.to_string())?;
        let url = Url::parse(url.trim()).map_err(|_| "Enter a direct HTTPS ZIP URL.")?;
        if url.scheme() != "https" || url.host().is_none() || !url.username().is_empty() || url.password().is_some() {
            return Err("Enter a direct HTTPS ZIP URL without embedded credentials.".into());
        }
        let root = library(&app).map_err(|error| error.to_string())?;
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let path = root.join(".download.zip");
        let result = (|| -> io::Result<()> {
            let agent = ureq::Agent::config_builder()
                .https_only(true)
                .timeout_connect(Some(Duration::from_secs(30)))
                .timeout_recv_body(Some(Duration::from_secs(3 * 60 * 60)))
                .build()
                .new_agent();
            let mut response = agent.get(url.as_str()).call()
                .map_err(|_| io::Error::other("Could not download from this HTTPS URL."))?;
            let total = response.body().content_length();
            if total.is_some_and(|size| size > MAX_ARCHIVE_BYTES) {
                return Err(io::Error::other("Archive exceeds the import size limit."));
            }
            let mut input = response.body_mut().as_reader();
            let mut header = [0; 4];
            if input.read_exact(&mut header).is_err() || header != *b"PK\x03\x04" {
                return Err(io::Error::other("The link did not return a ZIP file."));
            }
            let mut file = File::create(&path)?;
            file.write_all(&header)?;
            let mut received = header.len() as u64;
            let mut previous = Instant::now();
            let mut buffer = vec![0; 128 * 1024];
            loop {
                let count = input.read(&mut buffer)?;
                if count == 0 {
                    break;
                }
                received += count as u64;
                if received > MAX_ARCHIVE_BYTES {
                    return Err(io::Error::other("Archive exceeds the import size limit."));
                }
                file.write_all(&buffer[..count])?;
                if previous.elapsed() >= Duration::from_millis(250) {
                    let _ = on_progress.send(DownloadProgress { received, total, installing: false });
                    previous = Instant::now();
                }
            }
            if total.is_some_and(|size| size != received) {
                return Err(io::Error::other("The archive download was incomplete."));
            }
            drop(file);
            let _ = on_progress.send(DownloadProgress { received, total, installing: true });
            install_zip(File::open(&path)?, &root)
        })();
        let _ = fs::remove_file(path);
        result.map_err(|error| error.to_string())?;
        Ok(discover(&app))
    })
    .await
    .map_err(|error| error.to_string())?
}

fn install_zip(mut file: File, library: &Path) -> io::Result<()> {
    // Stage on the same filesystem as the live library so installation can use rename.
    let stage = fs::create_dir(library.join(".import")).map(|_| library.join(".import"))
        .or_else(|error| {
            if error.kind() != io::ErrorKind::AlreadyExists {
                return Err(error);
            }
            // A previous process may have stopped during replacement. Recover its backup first.
            let stage = library.join(".import");
            for entry in fs::read_dir(stage.join("backup")).into_iter().flatten() {
                let entry = entry?;
                let target = library.join(entry.file_name());
                if !target.exists() {
                    fs::rename(entry.path(), target)?;
                }
            }
            fs::remove_dir_all(&stage)?;
            fs::create_dir(&stage)?;
            Ok(stage)
        })?;
    let result = (|| {
        // Some document providers return a pipe. Copy those to disk so ZIP access can seek.
        if file.stream_position().is_err() {
            let path = stage.join("source.zip");
            let mut copy = File::create(&path)?;
            copy_limited(&mut file, &mut copy, MAX_ARCHIVE_BYTES)?;
            file = File::open(path)?;
        } else {
            file.seek(SeekFrom::Start(0))?;
        }
        let extracted = stage.join("files");
        fs::create_dir(&extracted)?;
        extract_zip(file, &extracted)?;
        let (source, id) = package_root(&extracted)?;
        let target = library.join(&id);
        let backup_dir = stage.join("backup");
        let backup = backup_dir.join(id);
        if target.exists() {
            fs::create_dir(&backup_dir)?;
            fs::rename(&target, &backup)?;
        }
        if let Err(error) = fs::rename(source, &target) {
            if backup.exists() {
                fs::rename(backup, target)?;
            }
            return Err(error);
        }
        Ok(())
    })();
    // Keep a backup on disk when replacement and rollback both fail.
    let has_unrestored_backup = fs::read_dir(stage.join("backup")).is_ok_and(|entries| {
        entries.flatten().any(|entry| !library.join(entry.file_name()).exists())
    });
    if !has_unrestored_backup {
        let _ = fs::remove_dir_all(stage);
    }
    result
}

fn copy_limited(input: &mut impl io::Read, output: &mut File, limit: u64) -> io::Result<u64> {
    let written = io::copy(&mut io::Read::take(input, limit + 1), output)?;
    if written > limit {
        return Err(io::Error::other("Archive exceeds the import size limit."));
    }
    Ok(written)
}

fn extract_zip(file: File, destination: &Path) -> io::Result<()> {
    let mut archive = zip::ZipArchive::new(file)?;
    if archive.len() > MAX_ENTRIES {
        return Err(io::Error::other("Archive has too many files."));
    }
    let mut remaining = MAX_ARCHIVE_BYTES;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.enclosed_name().ok_or_else(|| io::Error::other("Unsafe ZIP path."))?;
        if entry.is_symlink() {
            return Err(io::Error::other("ZIP symbolic links are unsupported."));
        }
        let path = destination.join(name);
        if entry.is_dir() {
            fs::create_dir_all(path)?;
        } else {
            if entry.size() > remaining {
                return Err(io::Error::other("Archive exceeds the import size limit."));
            }
            fs::create_dir_all(path.parent().unwrap_or(destination))?;
            remaining -= copy_limited(&mut entry, &mut File::create_new(path)?, remaining)?;
        }
    }
    Ok(())
}

fn package_root(extracted: &Path) -> io::Result<(PathBuf, String)> {
    let directories = fs::read_dir(extracted)?
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_type().ok()?.is_dir().then(|| entry.path()));
    let mut packages = std::iter::once(extracted.to_path_buf())
        .chain(directories)
        .filter_map(|path| package_id(&path).map(|id| (path, id)));
    match (packages.next(), packages.next()) {
        (Some(package), None) => Ok(package),
        _ => Err(io::Error::other(
            "ZIP exactly one supported mod, add-on or HOI4 game folder. Keep descriptor.mod for Workshop mods.",
        )),
    }
}

fn package_id(path: &Path) -> Option<String> {
    if path.join("hoi4.exe").is_file() && path.join("common").is_dir() {
        return Some(GAME.into());
    }
    let descriptor = super::decode(&path.join("descriptor.mod")).ok()?;
    let id = super::value(&descriptor, "remote_file_id");
    let known = super::KNOWN.iter().any(|(known_id, _)| *known_id == id)
        || matches!(id.as_str(), "2243912940" | "3350892196");
    known.then_some(id)
}
