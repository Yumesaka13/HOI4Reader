use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use walkdir::{DirEntry, WalkDir};

pub const TFR_CHILD: &str = "3609728809";
pub const TNO_CHILD: &str = "3256452254";
const TNO_CN: &str = "2243912940";

pub fn parent_id(root: &Path) -> Option<&'static str> {
    let id = root.file_name()?.to_str()?;
    match id {
        TFR_CHILD => Some("3350890356"),
        TNO_CHILD => Some("2438003901"),
        _ => None,
    }
}

pub fn parent(root: &Path) -> Option<PathBuf> {
    let id = parent_id(root)?;
    let sibling = root.parent()?.join(id);
    sibling.is_dir().then_some(sibling)
}

pub fn roots(root: &Path) -> Vec<PathBuf> {
    parent(root)
        .into_iter()
        .chain([root.to_path_buf()])
        .collect()
}

pub fn localisation_roots(root: &Path) -> Vec<PathBuf> {
    let mut sources = roots(root);
    let base = &sources[0];
    if base.file_name().is_some_and(|id| id == "2438003901") {
        let translation = base.with_file_name(TNO_CN);
        if translation.join("localisation").is_dir() {
            sources.insert(1, translation);
        }
    }
    sources
}

fn replaced(root: &Path) -> Vec<PathBuf> {
    super::decode(&root.join("descriptor.mod"))
        .map(|text| {
            super::script_values(&text, "replace_path")
                .into_iter()
                .map(|path| PathBuf::from(path.replace('\\', "/")))
                .collect()
        })
        .unwrap_or_default()
}

pub fn directory_exists(root: &Path, directory: &Path) -> bool {
    roots(root)
        .iter()
        .any(|folder| folder.join(directory).is_dir())
}

pub fn entries(root: &Path, directory: impl AsRef<Path>) -> impl Iterator<Item = DirEntry> {
    let directory = directory.as_ref();
    let layers = roots(root);
    if layers.len() == 1 {
        return WalkDir::new(root.join(directory))
            .sort_by_file_name()
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .collect::<Vec<_>>()
            .into_iter();
    }
    let exclusions = if layers.len() > 1 {
        replaced(root)
    } else {
        vec![]
    };
    let mut paths = BTreeMap::new();
    for layer in layers {
        for entry in WalkDir::new(layer.join(directory))
            .into_iter()
            .filter_map(Result::ok)
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let Ok(relative) = entry.path().strip_prefix(&layer) else {
                continue;
            };
            if layer != root && exclusions.iter().any(|prefix| relative.starts_with(prefix)) {
                continue;
            }
            paths.insert(relative.to_string_lossy().to_ascii_lowercase(), entry);
        }
    }
    let mut entries = paths.into_values().collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.path().starts_with(root));
    entries.into_iter()
}

pub fn file(root: &Path, relative: &str) -> Option<PathBuf> {
    let relative = relative.replace('\\', "/");
    let layers = roots(root);
    if let Some(path) = super::super_events::local_file(root, &relative) {
        return Some(path);
    }
    if layers.len() < 2
        || replaced(root)
            .iter()
            .any(|prefix| Path::new(&relative).starts_with(prefix))
    {
        return None;
    }
    super::super_events::local_file(&layers[0], &relative)
}

pub fn path(root: &Path, relative: &str) -> PathBuf {
    file(root, relative).unwrap_or_else(|| root.join(relative))
}

#[cfg(test)]
#[test]
fn tno_translation_is_inherited_and_child_files_override_the_base() -> std::io::Result<()> {
    use std::fs;
    let root = std::env::temp_dir().join(format!("hoi4reader-mod-layers-{}", std::process::id()));
    let base = root.join("2438003901");
    let translation = root.join(TNO_CN);
    let child = root.join(TNO_CHILD);
    let fixtures = [
        (base.join("events/shared.txt"), "base"),
        (base.join("events/base_only.txt"), "base only"),
        (child.join("events/shared.txt"), "child"),
        (
            base.join("localisation/english/test_l_english.yml"),
            "l_english:\nstory: \"Base\"\n",
        ),
        (
            translation.join("localisation/simp_chinese/test_l_simp_chinese.yml"),
            "l_simp_chinese:\nstory: \"Translated base\"\n",
        ),
        (
            child.join("localisation/simp_chinese/child_l_simp_chinese.yml"),
            "l_simp_chinese:\nchild: \"Child story\"\n",
        ),
    ];
    for (file, text) in &fixtures {
        fs::create_dir_all(file.parent().unwrap())?;
        fs::write(file, text)?;
    }
    assert_eq!(
        file(&child, "events/shared.txt"),
        Some(child.join("events/shared.txt"))
    );
    assert_eq!(entries(&child, "events").count(), 2);
    let chinese = super::localisation_for(&child, "simp_chinese", &mut vec![]);
    assert_eq!(
        chinese.get("story").map(String::as_str),
        Some("Translated base")
    );
    assert_eq!(
        chinese.get("child").map(String::as_str),
        Some("Child story")
    );
    assert_eq!(
        super::localisation_for(&child, "english", &mut vec![])
            .get("story")
            .map(String::as_str),
        Some("Base")
    );
    for (file, _) in fixtures {
        fs::remove_file(file)?;
    }
    for (path, dirs) in [
        (base, vec!["events", "localisation/english", "localisation"]),
        (
            child,
            vec!["events", "localisation/simp_chinese", "localisation"],
        ),
        (
            translation,
            vec!["localisation/simp_chinese", "localisation"],
        ),
    ] {
        for dir in dirs {
            fs::remove_dir(path.join(dir))?;
        }
        fs::remove_dir(path)?;
    }
    fs::remove_dir(root)
}
