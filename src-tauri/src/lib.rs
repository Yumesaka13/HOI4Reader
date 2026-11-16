mod index_cache;
mod mod_layers;
mod reader_notes;
mod super_events;
#[cfg(target_os = "android")]
mod android_library;

use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

const KNOWN: &[(&str, &str)] = &[
    ("2438003901", "The New Order: Last Days of Europe"),
    ("3350890356", "The Fire Rises"),
    (mod_layers::TFR_CHILD, "The Fire Rises: 同岛一命"),
    (mod_layers::TNO_CHILD, "The New Order: 道阻且长"),
];
#[derive(Serialize, Clone)]
pub struct ModInfo {
    pub name: String,
    pub path: String,
    pub workshop_id: String,
    pub image_path: Option<String>,
    pub parent_name: Option<String>,
}
#[derive(Deserialize, Serialize, Clone)]
pub struct Country {
    pub tag: String,
    pub name: String,
    pub flag_image: Option<String>,
    pub chapter_count: usize,
    pub story_count: usize,
    pub major_order: Option<usize>,
    pub chapter_indices: Vec<usize>,
}
#[derive(Deserialize, Serialize, Clone)]
struct CountryName {
    tag: String,
    name: String,
    major_order: Option<usize>,
}
#[derive(Deserialize, Serialize, Clone)]
pub struct Chapter {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selection_music: Vec<super_events::MusicEffect>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub music: Vec<super_events::MusicEffect>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bypass_condition: Option<ScriptCondition>,
    pub background_availability: Option<Vec<FlagCheck>>,
    pub ai_weight: f64,
    pub never_manually_available: bool,
    pub mission_time_changes: Vec<(String, i32)>,
    pub event_calls: Vec<(String, String)>,
    pub selection_calls: Vec<(String, String)>,
    pub selection_flags: Vec<FlagChange>,
    pub selection_missions: Vec<String>,
    pub selection_completions: Vec<FocusCompletion>,
    pub selection_routes: ConditionalRoutes,
    pub selection_foreign_events: Vec<ForeignEvent>,
    pub selection_broadcasts: Vec<(String, String)>,
    pub conditional_routes: ConditionalRoutes,
    pub mission_activations: Vec<String>,
    pub credits: Vec<String>,
    pub broadcast_calls: Vec<(String, String)>,
    pub foreign_events: Vec<ForeignEvent>,
    pub super_event: Option<super_events::Scene>,
    pub id: String,
    pub title: String,
    pub description: String,
    pub icon: String,
    pub image_available: bool,
    pub image_path: Option<String>,
    pub source: String,
    pub x: i32,
    pub y: i32,
    pub duration_days: u32,
    pub focus_tree_id: String,
    pub country_tags: Vec<String>,
    pub initial_tree: bool,
    pub branch_conditioned: bool,
    pub branch_flags: Option<Vec<FlagCheck>>,
    pub branch_condition: Option<ScriptCondition>,
    pub availability_condition: ScriptCondition,
    pub availability_flags: Vec<FlagCheck>,
    pub available_after_focuses: Vec<String>,
    pub flag_changes: Vec<FlagChange>,
    pub focus_tree_changes: Vec<String>,
    pub focus_completions: Vec<FocusCompletion>,
    pub branch_available_at_start: bool,
    pub completed_at_start: bool,
    pub relative_position_id: String,
    pub prerequisite_groups: Vec<Vec<String>>,
    pub mutually_exclusive_ids: Vec<String>,
}
#[derive(Deserialize, Serialize)]
pub struct ScanResult {
    inline_assets: BTreeMap<String, PathBuf>,
    text_colors: BTreeMap<String, String>,
    opening_notes: BTreeMap<String, Vec<reader_notes::OpeningNote>>,
    pub wartime_hooks: Vec<WartimeHook>,
    pub war_news_missions: HashMap<String, Vec<String>>,
    pub american_crisis_anchor: Option<String>,
    pub great_asian_war_start: Option<ForeignEvent>,
    pub great_asian_war_recipients: Vec<ForeignEvent>,
    pub great_asian_war_news: Option<(String, String)>,
    pub default_game_rules: HashMap<String, String>,
    pub start_date: Option<String>,
    pub initial_country_flags: HashMap<String, std::collections::BTreeSet<String>>,
    country_names: Vec<CountryName>,
    pub chapters: Vec<Chapter>,
    pub timeline: Vec<TimelineItem>,
    pub diagnostics: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct WartimeHook {
    id: String,
    war: String,
    kind: String,
    source: String,
    arms: Vec<WartimeArm>,
}

#[derive(Deserialize, Serialize)]
pub struct WartimeArm {
    condition: ScriptCondition,
    calls: Vec<ForeignEvent>,
    flags: Vec<FlagChange>,
}
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ForeignEvent {
    recipient: String,
    target: String,
    delay: String,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FocusCompletion {
    tag: Option<String>,
    id: String,
    bypass: bool,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct FlagCheck {
    flag: String,
    present: bool,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct FlagChange {
    flag: String,
    present: bool,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct StoryOption {
    #[serde(default, skip_serializing_if = "reader_notes::EffectPreview::is_empty")]
    effect_preview: reader_notes::EffectPreview,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub music: Vec<super_events::MusicEffect>,
    pub mission_time_changes: Vec<(String, i32)>,
    pub trigger_checks: Option<Vec<FlagCheck>>,
    pub trigger_condition: Option<ScriptCondition>,
    pub conditional_routes: ConditionalRoutes,
    pub mission_activations: Vec<String>,
    pub foreign_events: Vec<ForeignEvent>,
    pub credits: Vec<String>,
    pub event_calls: Vec<(String, String)>,
    pub flag_changes: Vec<FlagChange>,
    pub focus_tree_changes: Vec<String>,
    pub route_effects: Vec<String>,
    pub target_tags: HashMap<String, Vec<String>>,
    pub focus_completions: Vec<FocusCompletion>,
    pub super_event: Option<super_events::Scene>,
    pub id: String,
    pub text: String,
    pub next_ids: Vec<String>,
}
#[derive(Deserialize, Serialize, Clone)]
pub struct GuardedDecisionCall {
    target: String,
    delay: String,
    checks: Vec<FlagCheck>,
    deferred: bool,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ConditionalRoute {
    #[serde(skip_serializing_if = "Option::is_none")]
    owner: Option<String>,
    condition: ScriptCondition,
    trees: Vec<String>,
    completions: Vec<FocusCompletion>,
    calls: Vec<(String, String)>,
    foreign_events: Vec<ForeignEvent>,
    flags: Vec<FlagChange>,
    missions: Vec<String>,
    nested: ConditionalRoutes,
}

type ConditionalRoutes = Vec<Vec<ConditionalRoute>>;

#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "op", content = "args", rename_all = "snake_case")]
pub enum ScriptCondition {
    Check(FlagCheck),
    All(Vec<ScriptCondition>),
    Any(Vec<ScriptCondition>),
    Not(Box<ScriptCondition>),
    Unknown,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Mission {
    days: u32,
    shared_resolution: bool,
    complete: MissionEffect,
    timeout: MissionEffect,
    cancel: MissionEffect,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct MissionEffect {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    music: Vec<super_events::MusicEffect>,
    calls: Vec<(String, String)>,
    foreign_events: Vec<ForeignEvent>,
    conditional_routes: ConditionalRoutes,
    activated_missions: Vec<(String, String)>,
    flags: Vec<FlagChange>,
    trees: Vec<String>,
    completions: Vec<FocusCompletion>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TextVariant {
    text: String,
    flags: Vec<FlagCheck>,
    completed_focuses: Vec<String>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TimelineItem {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub music: Vec<super_events::MusicEffect>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision_end_music: Option<DecisionEndMusic>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manual_trigger: Option<ManualTrigger>,
    pub periodic_triggers: Vec<PeriodicTrigger>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autonomous_trigger: Option<AutonomousTrigger>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub focus_progress_trigger: Option<FocusProgressTrigger>,
    pub automatic_timeout: bool,
    pub verified_startup: Vec<ForeignEvent>,
    pub startup_trigger: Option<String>,
    pub decision_guarded_calls: Vec<GuardedDecisionCall>,
    pub hidden: bool,
    pub never_manually_available: bool,
    pub mission_time_changes: Vec<(String, i32)>,
    pub trigger_checks: Option<Vec<FlagCheck>>,
    pub conditional_routes: ConditionalRoutes,
    pub immediate_calls: Vec<(String, String)>,
    pub mission: Option<Mission>,
    pub decision_checks: Vec<FlagCheck>,
    pub decision_condition: Option<ScriptCondition>,
    pub decision_focuses: Vec<String>,
    pub mission_activations: Vec<String>,
    pub decision_event_calls: Vec<(String, String)>,
    pub title_variants: Vec<TextVariant>,
    pub description_variants: Vec<TextVariant>,
    pub credits: Vec<String>,
    pub broadcast: bool,
    pub broadcast_calls: Vec<(String, String)>,
    pub foreign_events: Vec<ForeignEvent>,
    pub flag_changes: Vec<FlagChange>,
    pub focus_tree_changes: Vec<String>,
    pub focus_completions: Vec<FocusCompletion>,
    pub super_event: Option<super_events::Scene>,
    pub id: String,
    pub kind: String,
    pub country_tag: String,
    pub title: String,
    pub description: String,
    pub date: Option<String>,
    pub date_after: bool,
    pub date_before: Option<String>,
    pub date_focuses: Vec<String>,
    pub date_country: Option<String>,
    pub timing: String,
    pub source: String,
    pub image_path: Option<String>,
    pub start_day: Option<i32>,
    pub start_timing: Option<String>,
    pub next_ids: Vec<String>,
    pub linked_from: Vec<String>,
    pub options: Vec<StoryOption>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct DecisionEndMusic {
    days: u32,
    effects: Vec<super_events::MusicEffect>,
    flags: Vec<FlagChange>,
    conditional_routes: ConditionalRoutes,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct PeriodicTrigger {
    owner: String,
    recipient: String,
    condition: ScriptCondition,
    interval: u32,
    delay: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AutonomousTrigger {
    owner: String,
    condition: ScriptCondition,
    mean_days: Option<u32>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct FocusProgressTrigger {
    focus: String,
    progress: f64,
    inclusive: bool,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct ManualTrigger {
    owner: String,
    earliest: Option<String>,
    earliest_inclusive: bool,
    latest: Option<String>,
    latest_inclusive: bool,
    focuses: Vec<String>,
    condition: ScriptCondition,
    prompt: Option<String>,
}
#[derive(Serialize)]
pub struct Discovery {
    pub game_path: String,
    pub workshop_path: String,
    pub mods: Vec<ModInfo>,
}
fn decode(p: &Path) -> Result<String, String> {
    let b = fs::read(p).map_err(|e| e.to_string())?;
    if let Ok(s) = std::str::from_utf8(&b) {
        return Ok(s.trim_start_matches('\u{feff}').to_owned());
    }
    let (enc, _, bad) = encoding_rs::WINDOWS_1252.decode(&b);
    if bad {
        Err("Unrecognized text encoding".into())
    } else {
        Ok(enc.into_owned())
    }
}
fn loc_parse(s: &str) -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    for line in s.trim_start_matches('\u{feff}').lines() {
        let l = line.trim();
        if l.is_empty() || l.starts_with('#') || l.starts_with("l_") {
            continue;
        }
        if let Some((k, v)) = l.split_once(':') {
            m.insert(
                k.trim().to_owned(),
                v.trim()
                    .trim_start_matches(|c: char| c.is_ascii_digit() || c == ' ' || c == '\"')
                    .trim_matches('\"')
                    .replace("\\n", "\n")
                    .replace("\\\"", "\""),
            );
        }
    }
    m
}
fn localisation_for(
    root: &Path,
    language: &str,
    diagnostics: &mut Vec<String>,
) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    let mut fallback = std::collections::HashSet::new();
    for layer in mod_layers::localisation_roots(root) {
        if layer.file_name().is_some_and(|id| id == "2243912940") && language != "simp_chinese" {
            continue;
        }
        let (english, selected) = localisation_for_layer(&layer, language, diagnostics);
        for (key, value) in english {
            fallback.insert(key.clone());
            result.insert(key, value);
        }
        for (key, value) in selected {
            fallback.remove(&key);
            result.insert(key, value);
        }
    }
    if !fallback.is_empty() {
        diagnostics.push(format!("{} texts use English fallback because '{language}' translations are missing.", fallback.len()));
    }
    result
}

fn localisation_for_layer(
    root: &Path,
    language: &str,
    diagnostics: &mut Vec<String>,
) -> (BTreeMap<String, String>, BTreeMap<String, String>) {
    let directory = root.join("localisation");
    if !directory.exists() {
        diagnostics.push(format!(
            "Localisation folder not found: {}",
            directory.display()
        ));
        return (BTreeMap::new(), BTreeMap::new());
    }
    let mut files = WalkDir::new(&directory)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.file_type().is_file()
                && entry
                    .path()
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("yml"))
        })
        .map(|entry| entry.path().to_path_buf())
        .collect::<Vec<_>>();
    files.sort_by(|a, b| {
        let replace = |p: &Path| p.strip_prefix(&directory).is_ok_and(|p| p.components().any(|part| part.as_os_str() == "replace"));
        (replace(a), a).cmp(&(replace(b), b))
    });
    let requested = language.to_ascii_lowercase();
    let wants_english = requested != "english";
    let mut english = BTreeMap::new();
    let mut selected = BTreeMap::new();
    for file in files {
        let name = file
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_ascii_lowercase();
        let is_selected = name.contains(&requested);
        let is_english = wants_english && name.contains("english");
        if !is_selected && !is_english {
            continue;
        }
        match decode(&file) {
            Ok(script) => {
                if is_english {
                    english.extend(loc_parse(&script));
                }
                if is_selected {
                    selected.extend(loc_parse(&script));
                }
            }
            Err(error) => diagnostics.push(format!("{}: {error}", file.display())),
        }
    }
    (english, selected)
}
fn name_for(id: &str, folder: &Path) -> String {
    if let Some((_, n)) = KNOWN.iter().find(|(x, _)| *x == id) {
        return (*n).to_string();
    }
    fs::read_to_string(folder.join("descriptor.mod"))
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.trim_start().starts_with("name="))
                .map(|l| {
                    l.trim()
                        .trim_start_matches("name=")
                        .trim_matches('"')
                        .to_owned()
                })
        })
        .unwrap_or_else(|| format!("Workshop {id}"))
}
fn mod_thumbnail(folder: &Path) -> Option<PathBuf> {
    let descriptor_picture = fs::read_to_string(folder.join("descriptor.mod"))
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                let (key, value) = line.trim().split_once('=')?;
                if key.trim() != "picture" {
                    return None;
                }
                let value = value.trim().trim_matches('"');
                let relative = Path::new(value);
                if relative.is_absolute()
                    || relative
                        .components()
                        .any(|part| matches!(part, std::path::Component::ParentDir))
                {
                    return None;
                }
                Some(folder.join(relative))
            })
        });
    descriptor_picture
        .into_iter()
        .chain(
            [
                "thumbnail.png",
                "thumbnail.jpg",
                "thumbnail.jpeg",
                "thumbnail.gif",
                "preview.png",
                "preview.jpg",
            ]
            .into_iter()
            .map(|name| folder.join(name)),
        )
        .find(|path| path.is_file())
}
fn scan_workshop(root: &Path) -> Vec<ModInfo> {
    let mut v = vec![];
    if let Ok(rd) = fs::read_dir(root) {
        for e in rd
            .flatten()
            .filter(|e| e.file_type().is_ok_and(|x| x.is_dir()))
        {
            let p = e.path();
            let id = e.file_name().to_string_lossy().to_string();
            if KNOWN.iter().any(|(known_id, _)| *known_id == id) {
                v.push(ModInfo {
                    name: name_for(&id, &p),
                    path: p.display().to_string(),
                    workshop_id: id,
                    image_path: mod_thumbnail(&p).or_else(|| mod_layers::parent(&p).and_then(|base| mod_thumbnail(&base))).map(|path| path.display().to_string()),
                    parent_name: mod_layers::parent_id(&p).map(|parent| name_for(parent, &p.with_file_name(parent))),
                })
            }
        }
    }
    v.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    v
}
#[tauri::command]
fn scan_workshop_dir(root: String) -> Vec<ModInfo> {
    scan_workshop(Path::new(&root))
}
#[cfg(not(target_os = "android"))]
fn steam_roots() -> Vec<PathBuf> {
    let mut out = vec![];
    #[cfg(windows)]
    {
        for d in b'C'..=b'Z' {
            for x in [
                format!("{}:\\Program Files (x86)\\Steam", d as char),
                format!("{}:\\Program Files\\Steam", d as char),
                format!("{}:\\Programs\\Steam", d as char),
                format!("{}:\\SteamLibrary", d as char),
            ] {
                let p = PathBuf::from(x);
                if p.join("steamapps").exists() {
                    out.push(p)
                }
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(h) = std::env::var_os("HOME") {
            out.push(PathBuf::from(h).join("Library/Application Support/Steam"));
        }
    }
    #[cfg(target_os = "linux")]
    {
        for p in ["/home", "/mnt", "/media"] {
            if Path::new(p).exists() {
                for e in WalkDir::new(p)
                    .max_depth(5)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter(|e| e.file_name() == "steamapps")
                {
                    if let Some(r) = e.path().parent() {
                        out.push(r.to_path_buf())
                    }
                }
            }
        }
    }
    out
}
#[cfg(not(target_os = "android"))]
fn discover_steam() -> Discovery {
    let roots = steam_roots();
    let (mut game, mut workshop, mut mods) = (String::new(), String::new(), vec![]);
    for root in roots {
        let mut libs = vec![root.clone()];
        if let Ok(v) = fs::read_to_string(root.join("steamapps/libraryfolders.vdf")) {
            for line in v.lines() {
                let t = line.trim();
                if t.starts_with("\"path\"") {
                    if let Some(p) = t.split('\"').nth(3) {
                        libs.push(PathBuf::from(p.replace("\\\\", "\\")));
                    }
                }
            }
        }
        for lib in libs {
            let g = lib.join("steamapps/common/Hearts of Iron IV");
            if game.is_empty() && g.join("common").exists() {
                game = g.display().to_string()
            }
            let w = lib.join("steamapps/workshop/content/394360");
            if w.exists() {
                workshop = w.display().to_string();
                mods.extend(scan_workshop(&w));
            }
        }
    }
    mods.sort_by(|a, b| a.path.cmp(&b.path));
    mods.dedup_by(|a, b| a.path == b.path);
    Discovery {
        game_path: game,
        workshop_path: workshop,
        mods,
    }
}

#[tauri::command]
fn discover(_app: tauri::AppHandle) -> Discovery {
    #[cfg(target_os = "android")]
    return android_library::discover(&_app);
    #[cfg(not(target_os = "android"))]
    discover_steam()
}
fn value(block: &str, key: &str) -> String {
    block
        .lines()
        .find_map(|line| {
            let line = clean_script_line(line).trim();
            line.strip_prefix(key)
                .and_then(|rest| rest.trim_start().strip_prefix('='))
                .map(|value| value.trim().trim_matches('"').to_string())
        })
        .unwrap_or_default()
}
fn clean_script_line(line: &str) -> &str {
    let mut quoted = false;
    let mut escaped = false;
    for (i, ch) in line.char_indices() {
        if escaped {
            escaped = false;
        } else if ch == '\\' && quoted {
            escaped = true;
        } else if ch == '"' {
            quoted = !quoted;
        } else if ch == '#' && !quoted {
            return &line[..i];
        }
    }
    line
}
fn top_value(block: &str, key: &str) -> String {
    let bytes = block.as_bytes();
    let Some(open) = block.find('{') else {
        return String::new();
    };
    let mut cursor = open + 1;
    let mut depth = 0usize;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'#' => {
                cursor = block[cursor..]
                    .find('\n')
                    .map_or(bytes.len(), |end| cursor + end + 1);
            }
            b'"' => cursor = skip_quoted(block, cursor),
            b'{' => {
                depth += 1;
                cursor += 1;
            }
            b'}' if depth == 0 => break,
            b'}' => {
                depth -= 1;
                cursor += 1;
            }
            byte if byte.is_ascii_whitespace() || byte == b',' => cursor += 1,
            _ => {
                let start = cursor;
                while cursor < bytes.len()
                    && !bytes[cursor].is_ascii_whitespace()
                    && !matches!(bytes[cursor], b'=' | b'{' | b'}' | b'#' | b',')
                {
                    cursor += 1;
                }
                if cursor == start {
                    cursor += 1;
                    continue;
                }
                let name = &block[start..cursor];
                while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                    cursor += 1;
                }
                if cursor >= bytes.len() || bytes[cursor] != b'=' {
                    continue;
                }
                cursor += 1;
                while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                    cursor += 1;
                }
                if cursor >= bytes.len() {
                    return String::new();
                }
                if cursor < bytes.len() && bytes[cursor] == b'{' {
                    depth += 1;
                    cursor += 1;
                    continue;
                }
                if name == key && depth == 0 {
                    let value_start = cursor;
                    let value_end = if bytes[cursor] == b'"' {
                        skip_quoted(block, cursor)
                    } else {
                        while cursor < bytes.len()
                            && !bytes[cursor].is_ascii_whitespace()
                            && !matches!(bytes[cursor], b'}' | b',' | b'#')
                        {
                            cursor += 1;
                        }
                        cursor
                    };
                    let raw = block[value_start..value_end].trim();
                    return raw.trim_matches('"').to_string();
                }
                if bytes[cursor] == b'"' {
                    cursor = skip_quoted(block, cursor);
                } else {
                    while cursor < bytes.len()
                        && !bytes[cursor].is_ascii_whitespace()
                        && !matches!(bytes[cursor], b'}' | b',' | b'#')
                    {
                        cursor += 1;
                    }
                }
            }
        }
    }
    String::new()
}
fn skip_quoted(text: &str, start: usize) -> usize {
    let bytes = text.as_bytes();
    let mut cursor = start + 1;
    let mut escaped = false;
    while cursor < bytes.len() {
        if escaped {
            escaped = false;
        } else if bytes[cursor] == b'\\' {
            escaped = true;
        } else if bytes[cursor] == b'"' {
            return cursor + 1;
        }
        cursor += 1;
    }
    bytes.len()
}
fn script_values(block: &str, key: &str) -> Vec<String> {
    let mut values = Vec::new();
    let bytes = block.as_bytes();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'#' => {
                cursor = block[cursor..]
                    .find('\n')
                    .map_or(bytes.len(), |end| cursor + end + 1);
            }
            b'"' => cursor = skip_quoted(block, cursor),
            b'{' => {
                cursor += 1;
            }
            b'}' => {
                cursor += 1;
            }
            byte if byte.is_ascii_whitespace() || byte == b',' => cursor += 1,
            _ => {
                let start = cursor;
                while cursor < bytes.len()
                    && !bytes[cursor].is_ascii_whitespace()
                    && !matches!(bytes[cursor], b'=' | b'{' | b'}' | b'#' | b',')
                {
                    cursor += 1;
                }
                if cursor == start {
                    cursor += 1;
                    continue;
                }
                let name = &block[start..cursor];
                while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                    cursor += 1;
                }
                if cursor >= bytes.len() || bytes[cursor] != b'=' {
                    continue;
                }
                cursor += 1;
                while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
                    cursor += 1;
                }
                if cursor >= bytes.len() {
                    break;
                }
                if bytes[cursor] == b'{' {
                    cursor += 1;
                    continue;
                }
                let value_start = cursor;
                let value_end = if bytes[cursor] == b'"' {
                    skip_quoted(block, cursor)
                } else {
                    while cursor < bytes.len()
                        && !bytes[cursor].is_ascii_whitespace()
                        && !matches!(bytes[cursor], b'}' | b',' | b'#')
                    {
                        cursor += 1;
                    }
                    cursor
                };
                if name == key {
                    let value = block[value_start..value_end].trim().trim_matches('"');
                    if !value.is_empty() {
                        values.push(value.to_string());
                    }
                }
                cursor = value_end;
            }
        }
    }
    values.sort();
    values.dedup();
    values
}
fn matching_brace_end(text: &str, open: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0i32;
    let mut quoted = false;
    let mut escaped = false;
    let mut comment = false;
    for i in open..bytes.len() {
        let b = bytes[i];
        if comment {
            if b == b'\n' {
                comment = false;
            }
            continue;
        }
        if escaped {
            escaped = false;
            continue;
        }
        if quoted && b == b'\\' {
            escaped = true;
            continue;
        }
        if b == b'"' {
            quoted = !quoted;
        } else if !quoted {
            if b == b'#' {
                comment = true;
            } else if b == b'{' {
                depth += 1;
            } else if b == b'}' {
                depth -= 1;
                if depth == 0 {
                    return Some(i + 1);
                }
            }
        }
    }
    None
}
fn blocks(text: &str, marker: &str) -> Vec<String> {
    let Some((name, _)) = marker.split_once('=') else {
        return Vec::new();
    };
    let name = name.trim();
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut cursor = 0usize;
    while cursor < bytes.len() {
        match bytes[cursor] {
            b'#' => {
                cursor = text[cursor..]
                    .find('\n')
                    .map_or(bytes.len(), |end| cursor + end + 1);
            }
            b'"' => cursor = skip_quoted(text, cursor),
            _ if bytes[cursor..].starts_with(name.as_bytes())
                && (cursor == 0
                    || !((bytes[cursor - 1] as char).is_ascii_alphanumeric()
                        || bytes[cursor - 1] == b'_'))
                && text
                    .as_bytes()
                    .get(cursor + name.len())
                    .map_or(true, |byte| {
                        !((*byte as char).is_ascii_alphanumeric() || *byte == b'_')
                    }) =>
            {
                let mut after_name = cursor + name.len();
                while after_name < bytes.len() && bytes[after_name].is_ascii_whitespace() {
                    after_name += 1;
                }
                if after_name >= bytes.len() || bytes[after_name] != b'=' {
                    cursor += name.len();
                    continue;
                }
                after_name += 1;
                while after_name < bytes.len() && bytes[after_name].is_ascii_whitespace() {
                    after_name += 1;
                }
                if after_name >= bytes.len() || bytes[after_name] != b'{' {
                    cursor = after_name;
                    continue;
                }
                if let Some(end) = matching_brace_end(text, after_name) {
                    out.push(text[cursor..end].to_string());
                    cursor = end;
                } else {
                    break;
                }
            }
            _ => cursor += 1,
        }
    }
    out
}
fn named_blocks(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut cursor = 0usize;
    while cursor < text.len() {
        let line_end = text[cursor..].find('\n').map_or(text.len(), |n| cursor + n);
        let line = clean_script_line(&text[cursor..line_end]).trim();
        if let Some((name, rhs)) = line.split_once('=') {
            if rhs.trim_start().starts_with('{') {
                let open = cursor + text[cursor..line_end].find('{').unwrap_or(0);
                if let Some(end) = matching_brace_end(text, open) {
                    out.push((name.trim().to_string(), text[cursor..end].to_string()));
                    cursor = end;
                    continue;
                }
            }
        }
        cursor = if line_end < text.len() {
            line_end + 1
        } else {
            text.len()
        };
    }
    out
}
fn source_country(path: &Path, tags: &std::collections::BTreeSet<String>) -> String {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_ascii_uppercase())
        .unwrap_or_default();
    stem.split(|c: char| !c.is_ascii_alphanumeric())
        .find(|part| tags.contains(*part))
        .unwrap_or_default()
        .to_string()
}
fn decision_links(scope: &str) -> Vec<(String, String)> {
    let mut links = Vec::new();
    for line in scope.lines() {
        let clean = clean_script_line(line).trim();
        if let Some((command, value)) = clean.split_once('=') {
            let command = command.trim();
            if matches!(
                command,
                "unlock_decision"
                    | "unlock_decision_tooltip"
                    | "activate_mission"
                    | "activate_mission_tooltip"
            ) {
                let target = value.trim().trim_matches('"').to_string();
                if !target.is_empty() {
                    let relation = if command.ends_with("_tooltip") {
                        "提示关联"
                    } else {
                        "脚本解锁"
                    };
                    links.push((target, relation.to_string()));
                }
            }
        }
    }
    links.sort();
    links.dedup();
    links
}
fn localized(loc: &BTreeMap<String, String>, key: &str) -> String {
    if key.is_empty() {
        return String::new();
    }
    loc.get(key)
        .cloned()
        .unwrap_or_else(|| format!("[Missing translation: {key}]"))
}
fn script_links(scope: &str) -> Vec<(String, String)> {
    let mut links = Vec::new();
    for command in ["country_event", "news_event", "unit_leader_event"] {
        if !scope.contains(command) {
            continue;
        }
        for call in blocks(scope, &format!("{command} = {{")) {
            let target = top_value(&call, "id");
            if !target.is_empty() {
                let delay = ["days", "hours", "random_days", "random_hours"]
                    .iter()
                    .filter_map(|key| {
                        let value = top_value(&call, key);
                        (!value.is_empty()).then_some(format!("{key} {value}"))
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                links.push((target, delay));
            }
        }
        for target in script_values(scope, command) {
            if target.contains('.') {
                links.push((target, String::new()));
            }
        }
    }
    links.sort();
    links.dedup();
    links
}
#[derive(Default)]
struct ScriptedEffects(HashMap<String, String>);

struct EventReferences<'a> {
    events: Vec<String>,
    effects: Vec<&'a str>,
}

impl ScriptedEffects {
    fn load(root: &Path) -> Self {
        let mut index = HashMap::new();
        for entry in mod_layers::entries(&root, "common/scripted_effects")
            .filter(|entry| {
                entry.file_type().is_file()
                    && entry
                        .path()
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("txt"))
            })
        {
            if let Ok(script) = decode(entry.path()) {
                index.extend(named_blocks(&script));
            }
        }
        Self(index)
    }

    fn expand_in_scope(&self, block: &str, depth: usize, remaining: &mut usize) -> String {
        if depth == 0 || *remaining == 0 {
            return block.to_string();
        }
        let Some(body) = block_body(block) else {
            return block.to_string();
        };
        let mut changes = Vec::new();
        for (key, value) in direct_statements(body) {
            let start = key.as_ptr() as usize - block.as_ptr() as usize;
            let end = value.as_ptr() as usize - block.as_ptr() as usize + value.len();
            if value == "yes" && *remaining > 0 && key != "TNO_fire_super_event" {
                if let Some(effect) = self.0.get(key) {
                    *remaining -= 1;
                    let expanded = self.expand_in_scope(effect, depth - 1, remaining);
                    if let Some(effects) = block_body(&expanded) {
                        changes.push((start..end, format!("hidden_effect = {{ {effects} }}")));
                    }
                }
            } else if value.starts_with('{')
                && (matches!(
                    key.to_ascii_lowercase().as_str(),
                    "hidden_effect" | "if" | "else_if" | "else"
                ) || (key.len() == 3
                    && key
                        .bytes()
                        .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())))
            {
                let expanded = self.expand_in_scope(value, depth - 1, remaining);
                if expanded != value {
                    let value_start = end - value.len();
                    changes.push((value_start..end, expanded));
                }
            }
        }
        let mut result = block.to_string();
        for (range, replacement) in changes.into_iter().rev() {
            result.replace_range(range, &replacement);
        }
        result
    }

    fn direct_event_references(&self, body: &str) -> EventReferences<'_> {
        static CALL: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
        let call = CALL.get_or_init(|| {
            regex::Regex::new(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(?:yes\b|\{)").unwrap()
        });
        let effects = body
            .lines()
            .map(clean_script_line)
            .flat_map(|line| call.captures_iter(line))
            .filter_map(|found| self.0.get_key_value(found.get(1)?.as_str()))
            .map(|(name, _)| name.as_str())
            .collect();
        EventReferences {
            events: script_links(body).into_iter().map(|(id, _)| id).collect(),
            effects,
        }
    }

    fn referenced_events<'a>(
        &'a self,
        root: &str,
        cache: &mut HashMap<&'a str, EventReferences<'a>>,
    ) -> std::collections::BTreeSet<String> {
        let direct = self.direct_event_references(root);
        let mut ids = direct.events.into_iter().collect::<std::collections::BTreeSet<_>>();
        let mut pending = direct.effects;
        let mut visited = std::collections::HashSet::new();
        while let Some(name) = pending.pop() {
            if !visited.insert(name) {
                continue;
            }
            let references = cache
                .entry(name)
                .or_insert_with(|| self.direct_event_references(&self.0[name]));
            ids.extend(references.events.iter().cloned());
            pending.extend(references.effects.iter().copied());
        }
        ids
    }
}

#[derive(Default)]
struct AutonomousTiming {
    after: Option<String>,
    before: Option<String>,
    tag: Option<String>,
    focuses: Vec<String>,
}

// Only a complete, understood top-level trigger can supply an autonomous date.
fn autonomous_date(block: &str) -> Option<AutonomousTiming> {
    if top_value(block, "is_triggered_only") == "yes"
        || top_value(block, "fire_only_once") != "yes"
        || !blocks(block, "mean_time_to_happen = {").is_empty()
    {
        return None;
    }
    static CLAUSE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let pattern = CLAUSE.get_or_init(|| regex::Regex::new(
        r"^(?:(tag|original_tag|has_completed_focus)\s*=\s*([\w.-]+)|date\s*([<>])\s*(\d{4}\.\d{1,2}\.\d{1,2}))(?:\s+|$)"
    ).unwrap());
    let trigger = direct_statements(block_body(block)?)
        .find_map(|(key, value)| (key == "trigger").then_some(value))?;
    let clean = block_body(trigger)?
        .lines()
        .map(clean_script_line)
        .collect::<Vec<_>>()
        .join(" ");
    let mut timing = AutonomousTiming::default();
    let mut rest = clean.trim();
    while !rest.is_empty() {
        let clause = pattern.captures(rest)?;
        match clause.get(1).map(|term| term.as_str()) {
            Some("has_completed_focus") => timing.focuses.push(clause[2].to_string()),
            Some("tag" | "original_tag")
                if timing.tag.is_none()
                    && clause[2].len() == 3
                    && clause[2]
                        .bytes()
                        .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit()) =>
            {
                timing.tag = Some(clause[2].to_string())
            }
            None if &clause[3] == ">" && timing.after.is_none() => {
                timing.after = Some(clause[4].to_string())
            }
            None if &clause[3] == "<" && timing.before.is_none() => {
                timing.before = Some(clause[4].to_string())
            }
            _ => return None,
        }
        rest = &rest[clause.get(0)?.end()..];
    }
    (timing.after.is_some() || !timing.focuses.is_empty()).then_some(timing)
}

fn manual_trigger(block: &str, country: &str) -> Option<ManualTrigger> {
    if top_value(block, "is_triggered_only") == "yes"
        || top_value(block, "fire_only_once") != "yes"
        || top_value(block, "hidden") == "yes"
        || !blocks(block, "mean_time_to_happen = {").is_empty()
    {
        return None;
    }
    static DATES: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let dates = DATES.get_or_init(|| {
        regex::Regex::new(r"^date\s*(>=|<=|>|<)\s*(\d{4}\.\d{1,2}\.\d{1,2})$").unwrap()
    });
    let trigger = direct_statements(block_body(block)?)
        .find(|(key, _)| *key == "trigger")?
        .1;
    let body = block_body(trigger)?;
    let owner = direct_statements(body)
        .find(|(key, value)| matches!(*key, "tag" | "original_tag") && value.len() == 3)
        .map_or(country, |(_, tag)| tag);
    if owner.is_empty() {
        return None;
    }
    let mut result = ManualTrigger {
        owner: owner.into(),
        earliest: None,
        earliest_inclusive: false,
        latest: None,
        latest_inclusive: false,
        focuses: Vec::new(),
        condition: ScriptCondition::Unknown,
        prompt: None,
    };
    let mut conditions = Vec::new();
    let mut scopes = vec![body];
    while let Some(scope) = scopes.pop() {
        for (key, value) in direct_statements(scope) {
            match key {
                "AND" | "hidden_trigger" => scopes.push(block_body(value)?),
                key if key == owner && value.starts_with('{') => scopes.push(block_body(value)?),
                "has_completed_focus" if !value.starts_with('{') => {
                    result.focuses.push(value.trim_matches('"').into())
                }
                "date" => {
                    let end = value.as_ptr() as usize - key.as_ptr() as usize + value.len();
                    let start = key.as_ptr() as usize - scope.as_ptr() as usize;
                    let text = scope.get(start..start + end)?;
                    let date = dates.captures(text)?;
                    let parts = date[2]
                        .split('.')
                        .map(str::parse::<u16>)
                        .collect::<Result<Vec<_>, _>>()
                        .ok()?;
                    if !(1..=12).contains(&parts[1]) || !(1..=31).contains(&parts[2]) {
                        return None;
                    }
                    if date[1].starts_with('>') && result.earliest.is_none() {
                        result.earliest = Some(date[2].into());
                        result.earliest_inclusive = &date[1] == ">=";
                    } else if date[1].starts_with('<') && result.latest.is_none() {
                        result.latest = Some(date[2].into());
                        result.latest_inclusive = &date[1] == "<=";
                    } else {
                        return None;
                    }
                }
                "has_war" if matches!(value, "yes" | "no") => {
                    result.prompt.get_or_insert_with(|| {
                        if value == "yes" {
                            "此时是否仍处于战争中？".into()
                        } else {
                            "此时是否已结束战争？".into()
                        }
                    });
                    conditions.push(ScriptCondition::Unknown);
                }
                "controls_state" | "owns_state" if !value.starts_with('{') => {
                    result.prompt.get_or_insert_with(|| {
                        format!(
                            "此时是否{}脚本指定的地区（地区编号 {}）？",
                            if key == "controls_state" {
                                "控制"
                            } else {
                                "拥有"
                            },
                            value
                        )
                    });
                    conditions.push(ScriptCondition::Unknown);
                }
                "has_stability" | "surrender_progress" if !value.starts_with('{') => {
                    let operator = &scope[(key.as_ptr() as usize - scope.as_ptr() as usize
                        + key.len())
                        ..(value.as_ptr() as usize - scope.as_ptr() as usize)]
                        .trim();
                    let term = match *operator {
                        ">" => "高于",
                        ">=" => "至少",
                        "<" => "低于",
                        "<=" => "至多",
                        _ => "达到",
                    };
                    if let Ok(number) = value.parse::<f64>() {
                        result.prompt.get_or_insert_with(|| {
                            format!(
                                "此时{}是否{} {}%？",
                                if key == "has_stability" {
                                    "国家稳定度"
                                } else {
                                    "投降进度"
                                },
                                term,
                                number * 100.0
                            )
                        });
                    }
                    conditions.push(ScriptCondition::Unknown);
                }
                "OR" | "NOT" | "NOR" if value.contains("date") => return None,
                _ => conditions.push(script_condition_in_country(
                    &format!("{key} = {value}"),
                    8,
                    owner,
                )),
            }
        }
    }
    if (result.earliest.is_none() && result.focuses.is_empty()) || conditions.is_empty() {
        return None;
    }
    result.condition = ScriptCondition::All(conditions);
    Some(result)
}

fn autonomous_trigger(block: &str, country: &str) -> Option<AutonomousTrigger> {
    if country.is_empty()
        || top_value(block, "is_triggered_only") == "yes"
        || top_value(block, "fire_only_once") != "yes"
    {
        return None;
    }
    let entries = direct_statements(block_body(block)?).collect::<Vec<_>>();
    let trigger = entries.iter().find(|(key, _)| *key == "trigger")?.1;
    let body = block_body(trigger)?;
    let owner = direct_statements(body)
        .find(|(key, value)| matches!(*key, "tag" | "original_tag") && value.len() == 3)
        .map_or(country, |(_, tag)| tag);
    let condition = script_condition_in_country(body, 8, owner);
    fn supported(condition: &ScriptCondition) -> Option<bool> {
        match condition {
            ScriptCondition::Unknown => None,
            ScriptCondition::Check(check) => Some(
                check.flag.starts_with("focus:")
                    || check.flag.starts_with("global:")
                    || (!check.flag.contains(':') && check.flag != "is_ai"),
            ),
            ScriptCondition::All(terms) | ScriptCondition::Any(terms) => terms
                .iter()
                .try_fold(false, |state, term| Some(state | supported(term)?)),
            ScriptCondition::Not(term) => supported(term),
        }
    }
    // A bare country tag is insufficient for countries created during play.
    if supported(&condition) != Some(true) {
        return None;
    }
    let mean_days = if let Some((_, mean)) = entries
        .iter()
        .find(|(key, _)| *key == "mean_time_to_happen")
    {
        let mut terms = direct_statements(block_body(mean)?);
        let (unit, value) = terms.next()?;
        if unit != "days" || terms.next().is_some() {
            return None;
        }
        Some(value.parse().ok()?)
    } else {
        None
    };
    Some(AutonomousTrigger {
        owner: owner.to_string(),
        condition,
        mean_days,
    })
}

fn focus_progress_trigger(block: &str) -> Option<FocusProgressTrigger> {
    if top_value(block, "is_triggered_only") == "yes" || top_value(block, "fire_only_once") != "yes"
    {
        return None;
    }
    let fields = direct_statements(block_body(block)?).collect::<Vec<_>>();
    if fields.iter().any(|(key, _)| *key == "mean_time_to_happen") {
        return None;
    }
    let (_, trigger) = fields.iter().find(|(key, _)| *key == "trigger")?;
    let mut conditions = direct_statements(block_body(trigger)?);
    let ("focus_progress", condition) = conditions.next()? else {
        return None;
    };
    if conditions.next().is_some() {
        return None;
    }
    static PROGRESS: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let pattern = PROGRESS.get_or_init(|| {
        regex::Regex::new(r"^\s*focus\s*=\s*([\w.-]+)\s+progress\s*(>=|>)\s*([0-9.]+)\s*$").unwrap()
    });
    let body = block_body(condition)?
        .lines()
        .map(clean_script_line)
        .collect::<Vec<_>>()
        .join(" ");
    let captures = pattern.captures(&body)?;
    let progress = captures[3].parse::<f64>().ok()?;
    (0.0..1.0)
        .contains(&progress)
        .then(|| FocusProgressTrigger {
            focus: captures[1].to_string(),
            progress,
            inclusive: &captures[2] == ">=",
        })
}

fn timing_label(block: &str) -> (Option<String>, String) {
    let date = top_value(block, "date");
    if date.split('.').count() == 3 && date.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return (Some(date.clone()), format!("脚本日期 {date}"));
    }
    if top_value(block, "is_triggered_only") == "yes" {
        (None, "由其他脚本触发；没有固定日期".into())
    } else if !value(block, "mean_time_to_happen").is_empty() {
        (None, "满足脚本条件后随机触发；日期不能静态确定".into())
    } else if !value(block, "trigger").is_empty() {
        (None, "满足脚本条件时触发；具体日期不能静态确定".into())
    } else {
        (None, "脚本未声明可确认的触发时间".into())
    }
}
fn icon_path(root: &Path, icon: &str) -> Option<PathBuf> {
    let wanted = icon_name(icon);
    for e in mod_layers::entries(&root, "gfx/interface/goals")
        .filter(|e| {
            e.file_type().is_file()
                && e.path()
                    .extension()
                    .is_some_and(|x| x.eq_ignore_ascii_case("png"))
        })
    {
        if e.path()
            .file_stem()
            .is_some_and(|x| x.to_string_lossy().eq_ignore_ascii_case(wanted))
        {
            return Some(e.path().to_path_buf());
        }
    }
    None
}
fn icon_name(icon: &str) -> &str {
    icon.trim_start_matches("GFX_").trim_start_matches("focus_")
}
fn icon_index(root: &Path) -> HashMap<String, PathBuf> {
    let mut index = HashMap::new();
    for e in mod_layers::entries(&root, "gfx/interface/goals")
        .filter(|e| {
            e.file_type().is_file()
                && e.path()
                    .extension()
                    .is_some_and(|x| x.eq_ignore_ascii_case("png"))
        })
    {
        if let Some(stem) = e.path().file_stem() {
            index
                .entry(stem.to_string_lossy().to_lowercase())
                .or_insert_with(|| e.path().to_path_buf());
        }
    }
    index
}
fn event_image_index(root: &Path) -> HashMap<String, PathBuf> {
    let mut index = HashMap::new();
    for entry in mod_layers::entries(&root, "gfx/event_pictures")
        .filter(|entry| {
            entry.file_type().is_file()
                && entry.path().extension().is_some_and(|extension| {
                    ["png", "jpg", "jpeg", "gif", "webp"]
                        .iter()
                        .any(|supported| extension.eq_ignore_ascii_case(supported))
                })
        })
    {
        if let Some(stem) = entry.path().file_stem() {
            index
                .entry(stem.to_string_lossy().to_lowercase())
                .or_insert_with(|| entry.path().to_path_buf());
        }
    }
    for entry in mod_layers::entries(&root, "interface")
    {
        if !entry.file_type().is_file()
            || !entry
                .path()
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("gfx"))
        {
            continue;
        }
        let Ok(script) = decode(entry.path()) else {
            continue;
        };
        if !script.contains("event_pictures") {
            continue;
        }
        for kind in ["spriteType = {", "SpriteType = {"] {
            for sprite in blocks(&script, kind) {
                let name = top_value(&sprite, "name");
                let mut texture = top_value(&sprite, "texturefile");
                if texture.is_empty() {
                    texture = top_value(&sprite, "textureFile");
                }
                let texture = texture.replace('\\', "/");
                if name.is_empty()
                    || !texture
                        .to_ascii_lowercase()
                        .starts_with("gfx/event_pictures/")
                {
                    continue;
                }
                if let Some(path) = mod_layers::file(root, &texture) {
                    if path.extension().is_some_and(|ext| {
                        ["png", "jpg", "jpeg", "gif", "webp"]
                            .iter()
                            .any(|supported| ext.eq_ignore_ascii_case(supported))
                    }) {
                        index.insert(icon_name(&name).to_lowercase(), path);
                    }
                }
            }
        }
    }
    index
}
// Only same-scope commands and hidden_effect wrappers are unconditional.
fn effect_commands(block: &str) -> Vec<(&str, &str)> {
    let mut commands = Vec::new();
    for (command, value) in direct_statements(block_body(block).unwrap_or_default()) {
        if command == "hidden_effect" {
            commands.extend(effect_commands(value));
        } else {
            commands.push((command, value));
        }
    }
    commands
}

fn mission_activations(block: &str) -> Vec<String> {
    let commands = effect_commands(block);
    let mut missions = commands
        .iter()
        .copied()
        .into_iter()
        .filter(|(key, value)| *key == "activate_mission" && !value.starts_with('{'))
        .map(|(_, value)| value.trim_matches('"').to_string())
        .collect::<Vec<_>>();
    for (_, scope) in commands
        .into_iter()
        .filter(|(key, value)| *key == "ZZZ" && value.starts_with('{'))
    {
        missions.extend(
            mission_activations(scope)
                .into_iter()
                .filter(|id| shared_war_resolution(id)),
        );
    }
    missions
}

fn shared_war_resolution(id: &str) -> bool {
    matches!(id, "second_nato_war_mission" | "third_euro_war_mission")
}

fn mission_time_changes(block: &str) -> Vec<(String, i32)> {
    effect_commands(block)
        .into_iter()
        .filter(|(key, value)| *key == "add_days_mission_timeout" && value.starts_with('{'))
        .filter_map(|(_, value)| {
            let id = top_value(value, "mission");
            let days = top_value(value, "days").parse().ok()?;
            (!id.is_empty()).then_some((id, days))
        })
        .collect()
}

fn never_manually_available(body: &str) -> bool {
    direct_statements(body).any(|(key, value)| match key {
        "always" => value == "no",
        "is_completed_by_event" => value == "yes",
        "hidden_trigger" | "AND" => block_body(value).is_some_and(never_manually_available),
        _ => false,
    })
}

fn mission_effect(
    block: &str,
    name: &str,
    owner: &str,
    scripted: &ScriptedEffects,
    audio: &super_events::Catalog,
) -> MissionEffect {
    let shared_war = shared_war_resolution(&top_value(block, "id"))
        || block
            .split_once('=')
            .is_some_and(|(id, _)| shared_war_resolution(id.trim()));
    let effects = blocks(block, &format!("{name} = {{"))
        .into_iter()
        .map(|effect| {
            let (depth, mut budget) = if shared_war { (12, 192) } else { (8, 32) };
            scripted.expand_in_scope(&effect, depth, &mut budget)
        })
        .collect::<Vec<_>>();
    MissionEffect {
        music: effects
            .iter()
            .flat_map(|effect| audio.music_effects(effect))
            .collect(),
        calls: effects
            .iter()
            .flat_map(|effect| local_event_calls(effect, owner))
            .collect(),
        foreign_events: effects
            .iter()
            .flat_map(|effect| scoped_event_calls(effect, owner))
            .collect(),
        conditional_routes: effects
            .iter()
            .flat_map(|effect| conditional_routes(effect, owner))
            .collect(),
        activated_missions: effects
            .iter()
            .flat_map(|effect| effect_commands(effect))
            .filter(|(scope, body)| {
                body.starts_with('{')
                    && scope.len() == 3
                    && scope
                        .bytes()
                        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
            })
            .flat_map(|(scope, body)| {
                mission_activations(body)
                    .into_iter()
                    .map(move |id| (scope.to_string(), id))
            })
            .collect(),
        flags: effects
            .iter()
            .flat_map(|effect| country_flag_changes(effect))
            .collect(),
        trees: effects
            .iter()
            .flat_map(|effect| focus_tree_changes(effect))
            .collect(),
        completions: effects
            .iter()
            .flat_map(|effect| block_focus_completions(effect))
            .collect(),
    }
}

fn trigger_checks(block: &str) -> Option<Vec<FlagCheck>> {
    let trigger = direct_statements(block_body(block)?)
        .find_map(|(key, value)| (key == "trigger").then_some(value))?;
    focus_flag_checks(block_body(trigger)?, true)
}

fn trigger_condition(block: &str) -> Option<ScriptCondition> {
    let trigger = direct_statements(block_body(block)?)
        .find_map(|(key, value)| (key == "trigger").then_some(value))?;
    Some(script_condition(block_body(trigger)?, 8))
}

fn script_condition(body: &str, depth: u8) -> ScriptCondition {
    script_condition_in_country(body, depth, "")
}

fn script_condition_in_country(body: &str, depth: u8, country: &str) -> ScriptCondition {
    use ScriptCondition::*;
    if depth == 0 {
        return Unknown;
    }
    All(direct_statements(body)
        .map(|(key, value)| match key {
            "check_variable"
                if top_value(value, "var") == "PRC_enviromental_progress"
                    && top_value(value, "compare") == "less_than_or_equals" =>
            {
                match top_value(value, "value").as_str() {
                    "0.3" => Check(FlagCheck {
                        flag: "reader:PRC_environment_30".into(),
                        present: true,
                    }),
                    "0.2" => Check(FlagCheck {
                        flag: "reader:PRC_environment_20".into(),
                        present: true,
                    }),
                    _ => Unknown,
                }
            }
            "AND" | "hidden_trigger" | "OR" | "NOT" | "NOR" => {
                let Some(body) = block_body(value) else {
                    return Unknown;
                };
                let inner = script_condition_in_country(body, depth - 1, country);
                match (key, inner) {
                    ("OR", All(terms)) => Any(terms),
                    ("NOR", All(terms)) => Not(Box::new(Any(terms))),
                    ("NOT", inner) => Not(Box::new(inner)),
                    (_, inner) => inner,
                }
            }
            "always" if value == "no" => Any(Vec::new()),
            "tag" | "original_tag" if value.len() == 3 => Check(FlagCheck {
                flag: format!("tag:{value}"),
                present: true,
            }),
            "is_ai" if matches!(value, "yes" | "no") => Check(FlagCheck {
                flag: "is_ai".into(),
                present: value == "yes",
            }),
            scope if !country.is_empty() && scope == country => block_body(value)
                .map(|body| script_condition_in_country(body, depth - 1, country))
                .unwrap_or(Unknown),
            scope
                if scope.len() == 3
                    && !matches!(scope, "ROOT" | "PREV" | "FROM" | "NOT" | "NOR" | "AND")
                    && scope
                        .bytes()
                        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()) =>
            {
                let Some(body) = block_body(value) else {
                    return Unknown;
                };
                let mut statements = direct_statements(body);
                match (statements.next(), statements.next()) {
                    (Some(("has_completed_focus", id)), None) if !id.starts_with('{') => {
                        Check(FlagCheck {
                            flag: format!("focus:{scope}:{}", id.trim_matches('"')),
                            present: true,
                        })
                    }
                    _ => Unknown,
                }
            }
            _ => match focus_flag_checks(&format!("{key} = {value}"), true) {
                Some(checks) => All(checks.into_iter().map(Check).collect()),
                None => Unknown,
            },
        })
        .collect())
}

fn conditional_routes(block: &str, owner: &str) -> ConditionalRoutes {
    conditional_routes_in_scope(block, owner, None, 12)
}

fn conditional_event_links(routes: &ConditionalRoutes) -> Vec<(String, String)> {
    routes
        .iter()
        .flatten()
        .flat_map(|route| {
            route
                .calls
                .iter()
                .cloned()
                .chain(
                    route
                        .foreign_events
                        .iter()
                        .map(|link| (link.target.clone(), link.delay.clone())),
                )
                .chain(conditional_event_links(&route.nested))
        })
        .collect()
}

fn conditional_routes_in_scope(
    block: &str,
    owner: &str,
    scope: Option<&str>,
    depth: u8,
) -> ConditionalRoutes {
    if depth == 0 {
        return Vec::new();
    }
    let mut routes = Vec::new();
    let mut arms: Vec<ConditionalRoute> = Vec::new();
    let flush = |routes: &mut ConditionalRoutes, arms: &mut Vec<ConditionalRoute>| {
        if arms.iter().any(|arm| {
            !arm.trees.is_empty()
                || !arm.completions.is_empty()
                || !arm.calls.is_empty()
                || !arm.foreign_events.is_empty()
                || !arm.flags.is_empty()
                || !arm.missions.is_empty()
                || !arm.nested.is_empty()
        }) {
            routes.push(std::mem::take(arms));
        } else {
            arms.clear();
        }
    };
    for (key, body) in direct_statements(block_body(block).unwrap_or_default()) {
        let branch = key.to_ascii_lowercase();
        if !matches!(branch.as_str(), "else_if" | "else") {
            flush(&mut routes, &mut arms);
        }
        if key == "hidden_effect" {
            routes.extend(conditional_routes_in_scope(body, owner, scope, depth - 1));
        } else if body.starts_with('{')
            && key.len() == 3
            && key
                .bytes()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
            && !matches!(key, "AND" | "NOT" | "NOR")
        {
            routes.extend(conditional_routes_in_scope(body, key, Some(key), depth - 1));
        }
        if branch != "if" && !(matches!(branch.as_str(), "else_if" | "else") && !arms.is_empty()) {
            continue;
        }
        let condition = if branch == "else" {
            ScriptCondition::All(Vec::new())
        } else {
            direct_statements(block_body(body).unwrap_or_default())
                .find_map(|(key, value)| (key == "limit").then_some(value))
                .and_then(block_body)
                .map(|value| script_condition_in_country(value, 8, owner))
                .unwrap_or(ScriptCondition::Unknown)
        };
        arms.push(ConditionalRoute {
            owner: scope.map(str::to_string),
            condition,
            trees: focus_tree_changes(body),
            completions: block_focus_completions(body),
            calls: local_event_calls(body, owner),
            foreign_events: scoped_event_calls(body, owner),
            flags: country_flag_changes(body),
            missions: mission_activations(body),
            nested: conditional_routes_in_scope(body, owner, scope, depth - 1),
        });
    }
    flush(&mut routes, &mut arms);
    routes
}

fn event_call(command: &str, value: &str) -> Option<(String, String)> {
    if value.starts_with('{') {
        script_links(&format!("{command} = {value}"))
            .into_iter()
            .next()
    } else {
        let id = value.trim_matches('"');
        id.contains('.').then(|| (id.to_string(), String::new()))
    }
}

fn direct_event_calls(block: &str, news_only: bool) -> Vec<(String, String)> {
    effect_commands(block)
        .into_iter()
        .filter(|(key, _)| {
            *key == "news_event"
                || (!news_only && matches!(*key, "country_event" | "unit_leader_event"))
        })
        .filter_map(|(key, value)| event_call(key, value))
        .collect()
}

const WARTIME_FLAGS: [&str; 6] = [
    "USA_american_civil_war_global",
    "PRC_ongoing_taiwan_war",
    "SOV_nato_war_begins_global",
    "second_nato_war_flag",
    "FAF_third_euro_war_global",
    "pan_asian_war_ongoing",
];

// A conjunctive war flag makes the hook eligible during this particular war.
// Keep OR/NOT clauses intact so they cannot supply a false connection.
fn wartime_flag(limit: &str) -> Option<&str> {
    for (key, value) in direct_statements(block_body(limit)?) {
        if key.eq_ignore_ascii_case("has_global_flag") && WARTIME_FLAGS.contains(&value) {
            return Some(value);
        }
        if matches!(key, "ROOT" | "FROM" | "AND" | "hidden_trigger") && value.starts_with('{') {
            if let Some(flag) = wartime_flag(value) {
                return Some(flag);
            }
        }
    }
    None
}

fn wartime_condition(limit: &str, war: &str) -> ScriptCondition {
    ScriptCondition::All(
        direct_statements(block_body(limit).unwrap_or_default())
            .map(|(key, value)| {
                if key.eq_ignore_ascii_case("has_global_flag") && value == war {
                    ScriptCondition::All(Vec::new())
                } else if matches!(key, "AND" | "hidden_trigger") && value.starts_with('{') {
                    wartime_condition(value, war)
                } else if matches!(key, "ROOT" | "FROM" | "FROM.FROM") {
                    ScriptCondition::Unknown
                } else if key.eq_ignore_ascii_case("NOT") || key.eq_ignore_ascii_case("NOR") {
                    script_condition_in_country(
                        &format!("{} = {value}", key.to_ascii_uppercase()),
                        8,
                        "",
                    )
                } else {
                    script_condition_in_country(&format!("{key} = {value}"), 8, "")
                }
            })
            .collect(),
    )
}

fn index_wartime_hook(
    kind: &str,
    script: &str,
    source: &str,
    effects: &ScriptedEffects,
    found: &mut Vec<WartimeHook>,
) {
    if !matches!(
        kind,
        "on_state_control_changed" | "on_capitulation" | "on_annex"
    ) {
        return;
    }
    for (effect_key, effect) in direct_statements(block_body(script).unwrap_or_default()) {
        if effect_key != "effect" {
            continue;
        }
        let branches = effect_commands(effect);
        let mut chain = Vec::new();
        let mut war = None;
        let flush =
            |arms: &mut Vec<WartimeArm>, war: &mut Option<&str>, found: &mut Vec<WartimeHook>| {
                if let Some(war) = war.take() {
                    if arms.iter().any(|arm| !arm.calls.is_empty()) {
                        found.push(WartimeHook {
                            id: format!("{source}:{kind}:{}", found.len()),
                            war: war.into(),
                            kind: kind.into(),
                            source: source.into(),
                            arms: std::mem::take(arms),
                        });
                    }
                }
                arms.clear();
            };
        for (key, body) in branches {
            let key = key.to_ascii_lowercase();
            if key != "else_if" && key != "else" {
                flush(&mut chain, &mut war, found);
            }
            if !matches!(key.as_str(), "if" | "else_if" | "else") {
                continue;
            }
            let limit = direct_statements(block_body(body).unwrap_or_default())
                .find_map(|(key, value)| (key == "limit").then_some(value));
            if key == "if" {
                war = limit.and_then(wartime_flag);
            }
            let Some(active_war) = war else {
                continue;
            };
            let expanded = effects.expand_in_scope(body, 8, &mut 32);
            let mut calls = scoped_event_calls(&expanded, "");
            calls.extend(
                direct_event_calls(&expanded, true)
                    .into_iter()
                    .map(|(target, delay)| ForeignEvent {
                        recipient: String::new(),
                        target,
                        delay,
                    }),
            );
            chain.push(WartimeArm {
                condition: limit.map_or(ScriptCondition::All(Vec::new()), |limit| {
                    wartime_condition(limit, active_war)
                }),
                calls,
                flags: country_flag_changes(&expanded)
                    .into_iter()
                    .filter(|flag| flag.flag.starts_with("global:"))
                    .collect(),
            });
        }
        flush(&mut chain, &mut war, found);
    }
}

fn local_event_calls(block: &str, owner: &str) -> Vec<(String, String)> {
    let mut calls = direct_event_calls(block, false);
    if owner.is_empty() {
        return calls;
    }
    for (scope, body) in effect_commands(block) {
        if scope == owner {
            calls.extend(direct_event_calls(body, false));
        } else if scope
            .strip_prefix(owner)
            .is_some_and(|suffix| suffix.starts_with('_'))
        {
            calls.extend(
                effect_commands(body)
                    .into_iter()
                    .filter(|(key, _)| *key == "unit_leader_event")
                    .filter_map(|(key, value)| event_call(key, value)),
            );
        }
    }
    calls
}

fn decision_event_calls(block: &str, owner: &str) -> Vec<(String, String)> {
    let mut calls = blocks(block, "complete_effect = {")
        .iter()
        .flat_map(|effect| local_event_calls(effect, owner))
        .collect::<Vec<_>>();
    if let Ok(days) = top_value(block, "days_remove").parse::<u32>() {
        for (target, delay) in blocks(block, "remove_effect = {")
            .iter()
            .flat_map(|effect| local_event_calls(effect, owner))
        {
            let delay = if delay.is_empty() {
                format!("days {days}")
            } else {
                format!("days {days}, {delay}")
            };
            calls.push((target, delay));
        }
    }
    calls
}

fn decision_guarded_calls(block: &str) -> Vec<GuardedDecisionCall> {
    let mut calls = Vec::new();
    for (effect_name, days) in [
        ("complete_effect", Some(0)),
        (
            "remove_effect",
            top_value(block, "days_remove").parse::<u32>().ok(),
        ),
    ] {
        let Some(days) = days else {
            continue;
        };
        for effect in blocks(block, &format!("{effect_name} = {{")) {
            for (_, branch) in effect_commands(&effect)
                .into_iter()
                .filter(|(key, _)| *key == "if")
            {
                let checks = direct_statements(block_body(branch).unwrap_or_default())
                    .find_map(|(key, value)| (key == "limit").then_some(value))
                    .and_then(block_body)
                    .and_then(|limit| focus_flag_checks(limit, true));
                let Some(checks) = checks else {
                    continue;
                };
                for (target, delay) in direct_event_calls(branch, false) {
                    let delay = if days == 0 {
                        delay
                    } else if delay.is_empty() {
                        format!("days {days}")
                    } else {
                        format!("days {days}, {delay}")
                    };
                    calls.push(GuardedDecisionCall {
                        target,
                        delay,
                        checks: checks.clone(),
                        deferred: effect_name == "remove_effect",
                    });
                }
            }
        }
    }
    calls
}

fn startup_trigger(block: &str) -> Option<String> {
    let trigger = direct_statements(block_body(block)?)
        .find_map(|(key, value)| (key == "trigger").then_some(value));
    let Some(trigger) = trigger else {
        return Some(String::new());
    };
    let mut tag = String::new();
    for (key, value) in direct_statements(block_body(trigger)?) {
        match key {
            "tag" | "original_tag"
                if !value.starts_with('{') && (tag.is_empty() || tag == value) =>
            {
                tag = value.to_string();
            }
            "always" if value == "yes" => {}
            _ => return None,
        }
    }
    Some(tag)
}

fn startup_recipient_condition(body: &str) -> bool {
    let entries = direct_statements(body).collect::<Vec<_>>();
    !entries.is_empty()
        && entries.into_iter().all(|(key, value)| match key {
            "tag" | "original_tag" => {
                value.len() == 3
                    && value
                        .bytes()
                        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
            }
            "OR" | "AND" => block_body(value).is_some_and(startup_recipient_condition),
            _ => false,
        })
}

fn startup_tag_matches(body: &str, tag: &str, any: bool) -> bool {
    let matches = direct_statements(body).map(|(key, value)| match key {
        "OR" | "AND" => {
            block_body(value).is_some_and(|body| startup_tag_matches(body, tag, key == "OR"))
        }
        _ => value == tag,
    });
    if any {
        matches.into_iter().any(|value| value)
    } else {
        matches.into_iter().all(|value| value)
    }
}

fn collect_startup_calls(
    block: &str,
    tags: &std::collections::BTreeSet<String>,
    recipient: Option<&str>,
    depth: u8,
    calls: &mut Vec<ForeignEvent>,
) {
    if depth == 0 {
        return;
    }
    for (key, body) in effect_commands(block) {
        if tags.contains(key) {
            collect_startup_calls(body, tags, Some(key), depth - 1, calls);
        } else if matches!(key, "every_country" | "every_possible_country" | "if") {
            let limit = direct_statements(block_body(body).unwrap_or_default())
                .find_map(|(key, value)| (key == "limit").then_some(value))
                .and_then(block_body);
            let Some(limit) = limit.filter(|body| startup_recipient_condition(body)) else {
                continue;
            };
            let resets_scope = key.starts_with("every_");
            for tag in tags.iter().filter(|tag| {
                (resets_scope || recipient.is_none_or(|country| country == tag.as_str()))
                    && startup_tag_matches(limit, tag, false)
            }) {
                collect_startup_calls(body, tags, Some(tag), depth - 1, calls);
            }
        } else if let Some(recipient) = recipient {
            if matches!(key, "country_event" | "news_event" | "unit_leader_event") {
                if let Some((target, delay)) = event_call(key, body) {
                    calls.push(ForeignEvent {
                        recipient: recipient.into(),
                        target,
                        delay,
                    });
                }
            }
        }
    }
}

fn collect_periodic_calls(
    block: &str,
    owner: &str,
    recipient: &str,
    interval: u32,
    tags: &std::collections::BTreeSet<String>,
    guards: &[ScriptCondition],
    depth: u8,
    found: &mut HashMap<String, Vec<PeriodicTrigger>>,
) {
    if depth == 0 {
        return;
    }
    let mut previous = Vec::new();
    for (key, value) in effect_commands(block) {
        if !matches!(key, "else_if" | "else") {
            previous.clear();
        }
        if matches!(key, "if" | "else_if" | "else") {
            if key != "if" && previous.is_empty() {
                continue;
            }
            let mut conditions = guards.to_vec();
            conditions.extend(
                previous
                    .iter()
                    .cloned()
                    .map(|arm| ScriptCondition::Not(Box::new(arm))),
            );
            if key != "else" {
                let condition = direct_statements(block_body(value).unwrap_or_default())
                    .find_map(|(key, value)| (key == "limit").then_some(value))
                    .and_then(block_body)
                    .map(|body| script_condition_in_country(body, 8, recipient))
                    .unwrap_or(ScriptCondition::Unknown);
                conditions.push(condition.clone());
                previous.push(condition);
            }
            collect_periodic_calls(
                value,
                owner,
                recipient,
                interval,
                tags,
                &conditions,
                depth - 1,
                found,
            );
        } else if tags.contains(key) {
            // Recipient scopes can dispatch events. Their local predicates
            // are not evaluated as though they belonged to the hook owner.
            if key == owner
                || !effect_commands(value)
                    .iter()
                    .any(|(key, _)| matches!(*key, "if" | "else_if" | "else"))
            {
                collect_periodic_calls(value, owner, key, interval, tags, guards, depth - 1, found);
            }
        } else if !guards.is_empty()
            && matches!(key, "country_event" | "news_event" | "unit_leader_event")
        {
            if let Some((id, delay)) = event_call(key, value) {
                found
                    .entry(id.to_ascii_lowercase())
                    .or_default()
                    .push(PeriodicTrigger {
                        owner: owner.into(),
                        recipient: recipient.into(),
                        interval,
                        delay,
                        condition: ScriptCondition::All(guards.to_vec()),
                    });
            }
        }
    }
}

fn scoped_event_calls(scope: &str, owner: &str) -> Vec<ForeignEvent> {
    let mut links = std::collections::BTreeSet::new();
    collect_scoped_event_calls(scope, owner, 12, &mut links);
    links.into_iter().collect()
}

fn collect_scoped_event_calls(
    scope: &str,
    owner: &str,
    depth: u8,
    links: &mut std::collections::BTreeSet<ForeignEvent>,
) {
    if depth == 0 {
        return;
    }
    for (recipient, body) in effect_commands(scope) {
        if recipient.len() != 3
            || !recipient
                .bytes()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
            || matches!(recipient, "AND" | "NOT" | "NOR")
            || !body.starts_with('{')
        {
            continue;
        }
        // Direct calls in the original country scope are already indexed as local.
        for (target, delay) in direct_event_calls(body, false)
            .into_iter()
            .filter(|_| recipient != owner)
        {
            links.insert(ForeignEvent {
                recipient: recipient.into(),
                target,
                delay,
            });
        }
        // Nested explicit scopes can dispatch to a third country or back to the
        // original owner. Their recipient is needed even in the latter case.
        collect_scoped_event_calls(body, "", depth - 1, links);
    }
}

fn ending_credits(block: &str, loc: &BTreeMap<String, String>) -> Vec<String> {
    static STYLE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let style = STYLE.get_or_init(|| regex::Regex::new(r"§[A-Za-z0-9!?]|£[A-Za-z0-9_]+").unwrap());
    effect_commands(block)
        .into_iter()
        .filter(|(key, _)| *key == "custom_effect_tooltip")
        .filter_map(|(_, key)| loc.get(key.trim_matches('"')))
        .filter_map(|value| {
            let text = style.replace_all(value, "");
            let lower = text.to_lowercase();
            (text.contains("感谢您游玩")
                || text.contains("感谢游玩")
                || text.contains("感谢你游玩")
                || text.contains("感謝您遊玩")
                || text.contains("感謝遊玩")
                || text.contains("感謝你遊玩")
                || lower.contains("thank you for playing")
                || lower.contains("thanks for playing")
                || (text.contains("プレイ") && text.contains("ありがとう")))
            .then(|| text.trim().to_string())
        })
        .collect()
}

fn focus_completions(scope: &str, tag: Option<&str>, depth: u8) -> Vec<FocusCompletion> {
    if depth == 0 {
        return Vec::new();
    }
    let mut results = Vec::new();
    for (name, value) in direct_statements(scope) {
        match name {
            "complete_national_focus" | "unlock_national_focus" => {
                let id = if value.starts_with('{') {
                    top_value(value, "focus")
                } else {
                    value.trim_matches('"').to_string()
                };
                if !id.is_empty() {
                    results.push(FocusCompletion {
                        tag: tag.map(str::to_string),
                        id,
                        bypass: name == "unlock_national_focus",
                    });
                }
            }
            "hidden_effect" => {
                if let Some(body) = block_body(value) {
                    results.extend(focus_completions(body, tag, depth - 1));
                }
            }
            recipient
                if recipient.len() == 3
                    && recipient
                        .bytes()
                        .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit()) =>
            {
                if let Some(body) = block_body(value) {
                    results.extend(focus_completions(body, Some(recipient), depth - 1));
                }
            }
            _ => {}
        }
    }
    results
}

fn option_target_tags(scope: &str) -> HashMap<String, Vec<String>> {
    static TAG: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let pattern = TAG.get_or_init(|| {
        regex::Regex::new(r"^\s*limit\s*=\s*\{\s*tag\s*=\s*([A-Z0-9]{3})\s*\}\s*$").unwrap()
    });
    let mut unscoped = scope.to_string();
    let mut targets: HashMap<String, Vec<String>> = HashMap::new();
    for branch in blocks(scope, "if = {") {
        let Some(limit) = blocks(&branch, "limit = {").into_iter().next() else {
            continue;
        };
        let Some(tag) = pattern.captures(&limit) else {
            continue;
        };
        if let Some(start) = unscoped.find(&branch) {
            unscoped.replace_range(start..start + branch.len(), "");
            for (target, _) in script_links(&branch) {
                targets
                    .entry(target.to_ascii_lowercase())
                    .or_default()
                    .push(tag[1].to_string());
            }
        }
    }
    if targets.is_empty() {
        return targets;
    }
    for (target, _) in script_links(&unscoped) {
        targets.remove(&target.to_ascii_lowercase());
    }
    targets
}

fn block_focus_completions(block: &str) -> Vec<FocusCompletion> {
    if !block.contains("complete_national_focus") && !block.contains("unlock_national_focus") {
        return Vec::new();
    }
    let (Some(open), Some(close)) = (block.find('{'), block.rfind('}')) else {
        return Vec::new();
    };
    focus_completions(&block[open + 1..close], None, 8)
}

fn add_focus_chapter(
    block: &str,
    source: &Path,
    focus_tree_id: &str,
    initial_tree: bool,
    country_tags: Vec<String>,
    start_state: &StartState,
    loc: &BTreeMap<String, String>,
    icons: &HashMap<String, PathBuf>,
    chapters: &mut Vec<Chapter>,
    edges: &mut Vec<(String, String, String)>,
    supers: &super_events::Catalog,
    effects: &ScriptedEffects,
) {
    let id = top_value(block, "id");
    if id.is_empty() {
        return;
    }
    let rewards = blocks(block, "completion_reward = {")
        .into_iter()
        .map(|reward| effects.expand_in_scope(&reward, 8, &mut 32))
        .collect::<Vec<_>>();
    let selection = direct_statements(block_body(block).unwrap_or_default())
        .filter(|(key, _)| *key == "select_effect")
        .map(|(_, effect)| effects.expand_in_scope(effect, 8, &mut 32))
        .collect::<Vec<_>>();
    for effect in &selection {
        for (target, delay) in script_links(effect) {
            edges.push((format!("focus-select:{id}"), target, delay));
        }
    }
    for reward in &rewards {
        for (target, delay) in script_links(&reward) {
            edges.push((format!("focus:{id}"), target, delay));
        }
        for (target, relation) in decision_links(&reward) {
            edges.push((format!("focus:{id} ({relation})"), target, String::new()));
        }
    }
    let icon = top_value(block, "icon");
    let image = if icon.is_empty() {
        None
    } else {
        icons.get(&icon_name(&icon).to_lowercase()).cloned()
    };
    let prerequisite_groups = blocks(block, "prerequisite = {")
        .into_iter()
        .map(|clause| script_values(&clause, "focus"))
        .filter(|ids| !ids.is_empty())
        .collect();
    let branch_block = blocks(block, "allow_branch = {");
    let branch_conditioned = !branch_block.is_empty();
    let empty_flags = std::collections::BTreeSet::new();
    let focus_tag = id.split('_').next().unwrap_or_default();
    let branch_flags = branch_block
        .first()
        .and_then(|branch| focus_flag_checks(block_body(branch)?, true));
    let branch_condition = branch_block
        .first()
        .and_then(|branch| block_body(branch))
        .map(|body| script_condition_in_country(body, 8, focus_tag));
    let start_flags = start_state.flags.get(focus_tag).unwrap_or(&empty_flags);
    let branch_available_at_start = branch_flags.as_ref().is_some_and(|checks| {
        checks
            .iter()
            .all(|check| start_flags.contains(&check.flag) == check.present)
    });
    let availability_flags = direct_statements(block_body(block).unwrap_or_default())
        .filter(|(key, _)| *key == "available")
        .filter_map(|(_, value)| focus_flag_checks(block_body(value)?, false))
        .flatten()
        .collect();
    let availability_condition = ScriptCondition::All(
        direct_statements(block_body(block).unwrap_or_default())
            .filter(|(key, _)| *key == "available")
            .filter_map(|(_, value)| block_body(value))
            .map(|body| script_condition_in_country(body, 8, focus_tag))
            .collect(),
    );
    let background_availability = direct_statements(block_body(block).unwrap_or_default())
        .filter(|(key, _)| *key == "available")
        .try_fold(Vec::new(), |mut checks, (_, value)| {
            checks.extend(focus_flag_checks(block_body(value)?, true)?);
            Some(checks)
        });
    let ai_weight = blocks(block, "ai_will_do = {")
        .first()
        .and_then(|weight| {
            ["factor", "base"]
                .iter()
                .find_map(|key| top_value(weight, key).parse::<f64>().ok())
        })
        .filter(|weight| weight.is_finite())
        .unwrap_or(1.0);
    let mutually_exclusive_ids = blocks(block, "mutually_exclusive = {")
        .into_iter()
        .flat_map(|clause| script_values(&clause, "focus"))
        .collect();
    let foreign_events = rewards
        .iter()
        .flat_map(|reward| scoped_event_calls(reward, focus_tag))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    chapters.push(Chapter {
        selection_music: selection
            .iter()
            .flat_map(|effect| supers.music_effects(effect))
            .collect(),
        music: rewards
            .iter()
            .flat_map(|effect| supers.music_effects(effect))
            .collect(),
        bypass_condition: (top_value(block, "enable_automatic_bypass") != "no")
            .then(|| {
                direct_statements(block_body(block).unwrap_or_default())
                    .find(|(key, _)| *key == "bypass")
                    .and_then(|(_, body)| block_body(body))
                    .filter(|body| direct_statements(body).next().is_some())
                    .map(|body| script_condition_in_country(body, 8, focus_tag))
            })
            .flatten(),
        background_availability,
        ai_weight,
        never_manually_available: direct_statements(block_body(block).unwrap_or_default())
            .filter(|(key, _)| *key == "available")
            .filter_map(|(_, value)| block_body(value))
            .any(never_manually_available),
        mission_time_changes: rewards
            .iter()
            .flat_map(|effect| mission_time_changes(effect))
            .collect(),
        event_calls: rewards
            .iter()
            .flat_map(|effect| local_event_calls(effect, focus_tag))
            .collect(),
        selection_calls: selection
            .iter()
            .flat_map(|effect| local_event_calls(effect, focus_tag))
            .collect(),
        selection_flags: selection
            .iter()
            .flat_map(|effect| country_flag_changes(effect))
            .collect(),
        selection_missions: selection
            .iter()
            .flat_map(|effect| mission_activations(effect))
            .collect(),
        selection_completions: selection
            .iter()
            .flat_map(|effect| block_focus_completions(effect))
            .collect(),
        selection_routes: selection
            .iter()
            .flat_map(|effect| conditional_routes(effect, focus_tag))
            .collect(),
        selection_foreign_events: selection
            .iter()
            .flat_map(|effect| scoped_event_calls(effect, focus_tag))
            .collect(),
        selection_broadcasts: selection
            .iter()
            .flat_map(|effect| direct_event_calls(effect, true))
            .collect(),
        conditional_routes: rewards
            .iter()
            .flat_map(|effect| conditional_routes(effect, focus_tag))
            .collect(),
        mission_activations: rewards
            .iter()
            .flat_map(|effect| mission_activations(effect))
            .collect(),
        credits: rewards
            .iter()
            .flat_map(|reward| ending_credits(reward, loc))
            .collect(),
        broadcast_calls: rewards
            .iter()
            .flat_map(|reward| direct_event_calls(reward, true))
            .collect(),
        foreign_events,
        super_event: supers.resolve(rewards.iter().map(String::as_str), loc),
        id: id.clone(),
        title: loc
            .get(&id)
            .cloned()
            .unwrap_or_else(|| format!("[Missing translation: {id}]")),
        description: loc.get(&format!("{id}_desc")).cloned().unwrap_or_default(),
        icon,
        image_available: image.is_some(),
        image_path: image.map(|path| path.display().to_string()),
        source: source.display().to_string(),
        x: top_value(block, "x").parse().unwrap_or(0),
        y: top_value(block, "y").parse().unwrap_or(0),
        duration_days: top_value(block, "cost")
            .parse::<f64>()
            .map(|weeks| (weeks.max(1.0) * 7.0).round() as u32)
            .unwrap_or(70),
        focus_tree_id: focus_tree_id.to_string(),
        country_tags,
        initial_tree,
        branch_conditioned,
        branch_flags,
        branch_condition,
        availability_condition,
        availability_flags,
        available_after_focuses: direct_statements(block_body(block).unwrap_or_default())
            .filter(|(key, _)| *key == "available")
            .flat_map(|(_, value)| direct_statements(block_body(value).unwrap_or_default()))
            .filter(|(key, value)| *key == "has_completed_focus" && !value.starts_with('{'))
            .map(|(_, value)| value.trim_matches('"').to_string())
            .collect(),
        focus_tree_changes: rewards
            .iter()
            .flat_map(|reward| focus_tree_changes(reward))
            .collect(),
        focus_completions: rewards
            .iter()
            .flat_map(|reward| block_focus_completions(reward))
            .collect(),
        flag_changes: rewards
            .iter()
            .flat_map(|reward| country_flag_changes(reward))
            .collect(),
        branch_available_at_start,
        completed_at_start: start_state.completed_focuses.contains(&id),
        relative_position_id: top_value(block, "relative_position_id"),
        prerequisite_groups,
        mutually_exclusive_ids,
    });
}
#[derive(Clone)]
struct FocusTreeInfo {
    id: String,
    tags: Vec<String>,
    static_country_selector: bool,
    default_tree: bool,
    shared_focuses: Vec<String>,
}
fn focus_tree_info(tree: &str) -> FocusTreeInfo {
    let country = blocks(tree, "country = {")
        .into_iter()
        .next()
        .unwrap_or_default();
    let positive_tags = blocks(&country, "modifier = {")
        .iter()
        .filter(|modifier| {
            top_value(modifier, "add")
                .parse::<f64>()
                .is_ok_and(|value| value > 0.0)
                || top_value(modifier, "factor")
                    .parse::<f64>()
                    .is_ok_and(|value| value > 0.0)
        })
        .flat_map(|modifier| direct_statements(block_body(modifier).unwrap_or_default()))
        .filter(|(key, value)| matches!(*key, "tag" | "original_tag") && value.len() == 3)
        .map(|(_, value)| value.to_string())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let static_country_selector = !positive_tags.is_empty()
        && country
            .lines()
            .filter_map(|line| clean_script_line(line).trim().split_once('='))
            .map(|(name, _)| name.trim())
            .all(|name| {
                matches!(
                    name,
                    "country" | "factor" | "base" | "modifier" | "add" | "tag" | "original_tag"
                )
            });
    FocusTreeInfo {
        id: top_value(tree, "id"),
        tags: if positive_tags.is_empty() {
            script_values(&country, "tag")
        } else {
            positive_tags
        },
        static_country_selector,
        default_tree: top_value(tree, "default") != "no",
        shared_focuses: script_values(tree, "shared_focus"),
    }
}
#[derive(Default)]
struct StartState {
    flags: HashMap<String, std::collections::BTreeSet<String>>,
    completed_focuses: std::collections::BTreeSet<String>,
}
fn collect_start_state(root: &Path, tags: &std::collections::BTreeSet<String>) -> StartState {
    let mut state = StartState::default();
    for entry in mod_layers::entries(&root, "history/countries")
        .filter(|entry| {
            entry.file_type().is_file()
                && entry
                    .path()
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("txt"))
        })
    {
        let tag = source_country(entry.path(), tags);
        if tag.is_empty() {
            continue;
        }
        let Ok(script) = decode(entry.path()) else {
            continue;
        };
        state
            .flags
            .entry(tag)
            .or_default()
            .extend(script_values(&script, "set_country_flag"));
        state
            .completed_focuses
            .extend(script_values(&script, "complete_national_focus"));
    }

    for entry in mod_layers::entries(&root, "common/on_actions")
        .filter(|entry| {
            entry.file_type().is_file()
                && entry
                    .path()
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("txt"))
        })
    {
        let Ok(script) = decode(entry.path()) else {
            continue;
        };
        for startup in blocks(&script, "on_startup = {") {
            for effect in blocks(&startup, "effect = {") {
                let Some(open) = effect.find('{') else {
                    continue;
                };
                let Some(close) = effect.rfind('}') else {
                    continue;
                };
                for (scope_name, scope) in named_blocks(&effect[open + 1..close]) {
                    if tags.contains(&scope_name) {
                        state
                            .flags
                            .entry(scope_name.clone())
                            .or_default()
                            .extend(script_values(&scope, "set_country_flag"));
                        state
                            .completed_focuses
                            .extend(script_values(&scope, "complete_national_focus"));
                    }
                    for condition in blocks(&scope, "if = {") {
                        let guarded_tags = blocks(&condition, "limit = {")
                            .into_iter()
                            .flat_map(|limit| script_values(&limit, "tag"))
                            .collect::<std::collections::BTreeSet<_>>();
                        for tag in guarded_tags.into_iter().filter(|tag| tags.contains(tag)) {
                            state
                                .flags
                                .entry(tag)
                                .or_default()
                                .extend(script_values(&condition, "set_country_flag"));
                            state
                                .completed_focuses
                                .extend(script_values(&condition, "complete_national_focus"));
                        }
                    }
                    if scope_name == "every_country" {
                        for condition in blocks(&scope, "if = {") {
                            let guarded_tags = blocks(&condition, "limit = {")
                                .into_iter()
                                .flat_map(|limit| script_values(&limit, "tag"))
                                .collect::<std::collections::BTreeSet<_>>();
                            for tag in guarded_tags.into_iter().filter(|tag| tags.contains(tag)) {
                                state
                                    .flags
                                    .entry(tag)
                                    .or_default()
                                    .extend(script_values(&condition, "set_country_flag"));
                                state
                                    .completed_focuses
                                    .extend(script_values(&condition, "complete_national_focus"));
                            }
                        }
                    }
                }
            }
        }
    }
    state
}
// Read direct assignments while keeping nested scopes intact. A missing or
// unsupported expression cannot be treated as an unconditional flag change.
fn direct_statements(mut body: &str) -> impl Iterator<Item = (&str, &str)> {
    static STATEMENT: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let pattern = STATEMENT.get_or_init(|| {
        regex::Regex::new(
            r#"^(?:\s+|#[^\n]*)*(\S+?)\s*(?:=|[<>]=?)\s*("(?:\\.|[^"\\])*"|\{|[^\s{}#"]+)"#,
        )
        .unwrap()
    });
    std::iter::from_fn(move || {
        let captures = pattern.captures(body)?;
        let key = captures.get(1)?.as_str();
        let value = captures.get(2)?;
        let end = if value.as_str() == "{" {
            matching_brace_end(body, value.start())?
        } else {
            value.end()
        };
        let result = (key, &body[value.start()..end]);
        body = &body[end..];
        Some(result)
    })
}

fn block_body(block: &str) -> Option<&str> {
    let start = block.find('{')? + 1;
    let end = block.rfind('}')?;
    block.get(start..end)
}

fn focus_flag_checks(body: &str, strict: bool) -> Option<Vec<FlagCheck>> {
    let mut checks = Vec::new();
    for (key, value) in direct_statements(body) {
        match key {
            "always" if value == "yes" => {}
            "has_country_flag" if !value.starts_with('{') => checks.push(FlagCheck {
                flag: value.trim_matches('"').to_string(),
                present: true,
            }),
            "has_completed_focus" if strict && !value.starts_with('{') => checks.push(FlagCheck {
                flag: format!("focus:{}", value.trim_matches('"')),
                present: true,
            }),
            "has_game_rule" if strict => {
                let rule = top_value(value, "rule");
                let option = top_value(value, "option");
                if rule.is_empty() || option.is_empty() {
                    return None;
                }
                checks.push(FlagCheck {
                    flag: format!("rule:{rule}={option}"),
                    present: true,
                });
            }
            "has_global_flag"
                if !value.starts_with('{')
                    && (strict
                        || matches!(
                            value,
                            "PRC_ongoing_taiwan_war"
                                | "PRC_taiwan_war_lost"
                                | "PRC_taiwan_war_won"
                                | "USA_american_civil_war_global"
                        )) =>
            {
                checks.push(FlagCheck {
                    flag: format!("global:{}", value.trim_matches('"')),
                    present: true,
                })
            }
            "hidden_trigger" | "AND" => {
                checks.extend(focus_flag_checks(block_body(value)?, strict)?);
            }
            "NOT" => {
                let negative = block_body(value).and_then(|body| focus_flag_checks(body, true));
                match negative.as_deref() {
                    Some([check]) => checks.push(FlagCheck {
                        flag: check.flag.clone(),
                        present: !check.present,
                    }),
                    _ if strict => return None,
                    _ => {}
                }
            }
            _ if strict => return None,
            _ => {}
        }
    }
    Some(checks)
}

fn event_text(
    entries: &[(&str, &str)],
    field: &str,
    loc: &BTreeMap<String, String>,
) -> (String, Vec<TextVariant>) {
    let mut fallback = String::new();
    let mut variants = Vec::new();
    for (_, value) in entries.iter().copied().filter(|(key, _)| *key == field) {
        if !value.starts_with('{') {
            if fallback.is_empty() {
                fallback = localized(loc, value.trim_matches('"'));
            }
            continue;
        }
        let key = top_value(value, "text");
        if key.is_empty() {
            continue;
        }
        let Some(trigger) = direct_statements(block_body(value).unwrap_or_default())
            .find_map(|(key, value)| (key == "trigger").then_some(value))
        else {
            if fallback.is_empty() {
                fallback = localized(loc, &key);
            }
            continue;
        };
        let mut flags = Vec::new();
        let mut completed_focuses = Vec::new();
        let supported =
            direct_statements(block_body(trigger).unwrap_or_default()).all(|(kind, value)| {
                match kind {
                    "has_completed_focus" if !value.starts_with('{') => {
                        completed_focuses.push(value.trim_matches('"').to_string());
                    }
                    "has_country_flag" | "has_global_flag" if !value.starts_with('{') => {
                        let flag = value.trim_matches('"');
                        flags.push(FlagCheck {
                            flag: if kind == "has_global_flag" {
                                format!("global:{flag}")
                            } else {
                                flag.into()
                            },
                            present: true,
                        });
                    }
                    _ => return false,
                }
                true
            });
        if supported {
            variants.push(TextVariant {
                text: localized(loc, &key),
                flags,
                completed_focuses,
            });
        }
    }
    (fallback, variants)
}

fn default_game_rules(root: &Path) -> HashMap<String, String> {
    let mut rules = HashMap::new();
    let mut files = mod_layers::entries(&root, "common/game_rules")
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .collect::<Vec<_>>();
    files.sort();
    for path in files {
        if let Ok(script) = decode(&path) {
            for (rule, body) in named_blocks(&script) {
                if let Some(default) = blocks(&body, "default = {").first() {
                    let value = top_value(default, "name");
                    if !value.is_empty() {
                        rules.insert(rule, value);
                    }
                }
            }
        }
    }
    rules
}

fn country_flag_changes(block: &str) -> Vec<FlagChange> {
    let mut changes = Vec::new();
    for (key, value) in direct_statements(block_body(block).unwrap_or_default()) {
        match key {
            "hidden_effect" => changes.extend(country_flag_changes(value)),
            "set_country_flag" | "clr_country_flag" | "set_global_flag" | "clr_global_flag"
                if !value.starts_with('{') =>
            {
                changes.push(FlagChange {
                    flag: if key.ends_with("global_flag") {
                        format!("global:{}", value.trim_matches('"'))
                    } else {
                        value.trim_matches('"').to_string()
                    },
                    present: key.starts_with("set_"),
                });
            }
            _ => {}
        }
    }
    changes
}
fn focus_tree_changes(block: &str) -> Vec<String> {
    let mut trees = Vec::new();
    for (key, value) in direct_statements(block_body(block).unwrap_or_default()) {
        match key {
            "hidden_effect" => trees.extend(focus_tree_changes(value)),
            "load_focus_tree" if !value.starts_with('{') => {
                trees.push(value.trim_matches('"').into())
            }
            "load_focus_tree" => {
                if let Some(tree) = direct_statements(block_body(value).unwrap_or_default())
                    .find_map(|(key, value)| (key == "tree").then_some(value))
                {
                    trees.push(tree.trim_matches('"').into());
                }
            }
            _ => {}
        }
    }
    trees
}

fn country_flag(root: &Path, tag: &str) -> Option<String> {
    if !tag.bytes().all(|byte| byte.is_ascii_alphanumeric()) {
        return None;
    }
    let tag = tag.to_ascii_uppercase();
    for directory in ["gfx/flags", "gfx/flags/medium", "flags"] {
        for extension in ["tga", "png"] {
            let Some(path) = mod_layers::file(root, &format!("{directory}/{tag}.{extension}")) else { continue; };
            let Ok(reader) = image::ImageReader::open(path) else {
                continue;
            };
            let Ok(flag) = reader.decode() else {
                continue;
            };
            let flag = if flag.width() > 160 || flag.height() > 96 {
                flag.thumbnail(160, 96)
            } else {
                flag
            };
            let mut png = std::io::Cursor::new(Vec::new());
            if flag.write_to(&mut png, image::ImageFormat::Png).is_ok() {
                return Some(format!(
                    "data:image/png;base64,{}",
                    STANDARD.encode(png.into_inner())
                ));
            }
        }
    }
    None
}

#[derive(Default)]
struct BookmarkIndex {
    start_date: Option<String>,
    major_tags: Vec<String>,
}

fn bookmark_index(root: &Path) -> BookmarkIndex {
    let mut dates = std::collections::BTreeSet::new();
    let mut defaults = std::collections::BTreeSet::new();
    let mut playable = Vec::new();
    for entry in mod_layers::entries(root, "common/bookmarks") {
        let path = entry.path();
        if !path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("txt"))
        {
            continue;
        }
        let Ok(script) = decode(&path) else { continue };
        for bookmark in blocks(&script, "bookmark = {") {
            let date = top_value(&bookmark, "date");
            let parts = date.split('.').take(3).collect::<Vec<_>>();
            if parts.len() != 3 || parts.iter().any(|part| part.parse::<u32>().is_err()) {
                continue;
            }
            let date = parts.join(".");
            dates.insert(date.clone());
            let is_default = top_value(&bookmark, "default") == "yes";
            if is_default {
                defaults.insert(date);
            }
            let mut seen = std::collections::HashSet::new();
            let tags = direct_statements(block_body(&bookmark).unwrap_or_default())
                .filter_map(|(tag, body)| {
                    let tag = tag.trim_matches('"');
                    (tag.len() == 3
                        && tag.bytes().all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
                        && body.starts_with('{')
                        && top_value(body, "minor") == "no"
                        && seen.insert(tag))
                        .then(|| tag.to_owned())
                })
                .collect::<Vec<_>>();
            if !tags.is_empty() {
                playable.push((is_default, tags));
            }
        }
    }
    let has_default = !defaults.is_empty();
    let start_date = if defaults.len() == 1 {
        defaults.into_iter().next()
    } else if dates.len() == 1 {
        dates.into_iter().next()
    } else {
        None
    };
    if has_default {
        playable.retain(|(is_default, _)| *is_default);
    }
    let major_tags = if playable.len() == 1 {
        playable.pop().unwrap().1
    } else {
        Vec::new()
    };
    BookmarkIndex { start_date, major_tags }
}

fn next_election_date(history: &str) -> Option<String> {
    let politics = named_blocks(history)
        .into_iter()
        .find(|(key, _)| key == "set_politics")?
        .1;
    if top_value(&politics, "elections_allowed") != "yes" {
        return None;
    }
    let last = top_value(&politics, "last_election");
    let mut parts = last.split('.').map(str::parse::<u32>);
    let (year, month, day) = (
        parts.next()?.ok()?,
        parts.next()?.ok()?,
        parts.next()?.ok()?,
    );
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let term = top_value(&politics, "election_frequency")
        .parse::<u32>()
        .ok()?;
    if term == 0 {
        return None;
    }
    let month_index = year
        .checked_mul(12)?
        .checked_add(month - 1)?
        .checked_add(term)?;
    let (year, month) = (month_index / 12, month_index % 12 + 1);
    let max_day = match month {
        2 if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    Some(format!("{year}.{month}.{}", day.min(max_day)))
}

// TFR reuses these statistical notices across countries. Keep the filter
// independent of localisation and leave other events in the same file intact.
fn is_shared_development_notice(path: &Path, id: &str) -> bool {
    path.file_name()
        .is_some_and(|name| name.eq_ignore_ascii_case("TFR_events_ZZZ_political.txt"))
        && id
            .strip_prefix("generic.")
            .and_then(|number| number.parse::<u8>().ok())
            .is_some_and(|number| (1..=18).contains(&number))
}

fn country_name<'a>(loc: &'a BTreeMap<String, String>, tag: &'a str) -> &'a str {
    loc.get(&format!("{tag}_DEF"))
        .or_else(|| loc.get(&format!("{tag}_ADJ")))
        .map_or(tag, String::as_str)
}

struct IndexInputs {
    loc: BTreeMap<String, String>,
    tags: std::collections::BTreeSet<String>,
    country_names: Vec<CountryName>,
    start_date: Option<String>,
    diagnostics: Vec<String>,
}

fn prepare_index(root: &Path, language: &str) -> IndexInputs {
    let mut diagnostics = vec![];
    let loc = localisation_for(root, language, &mut diagnostics);
    let bookmarks = bookmark_index(root);
    let mut tags = std::collections::BTreeSet::new();
    for k in loc.keys() {
        if let Some(t) = k.strip_suffix("_DEF").or_else(|| k.strip_suffix("_ADJ")) {
            if (2..=4).contains(&t.len()) && !t.contains('_') {
                tags.insert(t.to_string());
            }
        }
    }
    tags.extend(bookmarks.major_tags.iter().cloned());
    let country_names = tags.iter().map(|tag| CountryName {
        tag: tag.clone(),
        name: country_name(&loc, tag).to_owned(),
        major_order: bookmarks.major_tags.iter().position(|major| major == tag),
    }).collect();
    IndexInputs { loc, tags, country_names, start_date: bookmarks.start_date, diagnostics }
}

fn index_mod(root: PathBuf, language: &str, inputs: IndexInputs, game: Option<&Path>) -> ScanResult {
    let IndexInputs { loc, tags, country_names, start_date, mut diagnostics } = inputs;
    let supers = super_events::Catalog::load(&root, language, &loc, &mut diagnostics);
    let effects = ScriptedEffects::load(&root);
    let opening_notes = reader_notes::opening_notes(&root, &loc);
    let tooltip_templates = reader_notes::game_templates(game, language);
    let start_state = collect_start_state(&root, &tags);
    let dir = root.join("common/national_focus");
    let icons = icon_index(&root);
    let event_images = event_image_index(&root);
    let mut chapters = vec![];
    let mut edges: Vec<(String, String, String)> = Vec::new();
    if mod_layers::directory_exists(&root, Path::new("common/national_focus")) {
        let scripts = mod_layers::entries(&root, "common/national_focus")
            .filter(|e| {
                e.file_type().is_file()
                    && e.path()
                        .extension()
                        .is_some_and(|x| x.eq_ignore_ascii_case("txt"))
            })
            .map(|entry| entry.path().to_path_buf())
            .collect::<Vec<_>>();
        let scripts = scripts
            .into_iter()
            .filter_map(|source| decode(&source).ok().map(|script| (source, script)))
            .collect::<Vec<_>>();

        // A tree often references shared_focus blocks in a different file. Build a global
        // membership index first so those chapters inherit the correct tree and start status.
        let mut trees = Vec::new();
        for (_, script) in &scripts {
            trees.extend(
                blocks(script, "focus_tree = {")
                    .into_iter()
                    .map(|tree| focus_tree_info(&tree)),
            );
        }
        let mut initial_tree_ids = std::collections::BTreeSet::new();
        let mut static_tags = std::collections::BTreeSet::new();
        for tree in trees.iter().filter(|tree| tree.static_country_selector) {
            static_tags.extend(tree.tags.iter().cloned());
        }
        let mut ambiguous_tags = Vec::new();
        for tag in static_tags {
            let candidates = trees
                .iter()
                .filter(|tree| tree.static_country_selector && tree.tags.contains(&tag))
                .collect::<Vec<_>>();
            let named = candidates
                .iter()
                .copied()
                .filter(|tree| {
                    let id = tree.id.to_ascii_lowercase();
                    id.contains("initial_tree")
                        || id.contains("tree_initial")
                        || id.contains("main_tree")
                })
                .collect::<Vec<_>>();
            let defaults = candidates
                .iter()
                .copied()
                .filter(|tree| tree.default_tree)
                .collect::<Vec<_>>();
            let selected = if named.len() == 1 {
                named.first().copied()
            } else if named.is_empty() && candidates.len() == 1 {
                candidates.first().copied()
            } else if named.is_empty() && defaults.len() == 1 {
                defaults.first().copied()
            } else {
                ambiguous_tags.push(tag.clone());
                None
            };
            if let Some(tree) = selected {
                initial_tree_ids.insert(tree.id.clone());
            }
        }
        // These opening selectors contain negative gates for later campaigns.
        // Require the matching country and the actual opening root focus.
        for (tag, tree_id, root_focus) in [
            ("GER", "GER_tree_initial", "GER_convene_the_bundestag"),
            ("FRA", "FRA_initial_tree", "FRA_The_Opening_Volley"),
            ("USA", "USA_focus", "USA_state_of_the_union"),
        ] {
            if trees
                .iter()
                .any(|tree| tree.id == tree_id && tree.tags.iter().any(|country| country == tag))
                && scripts.iter().any(|(_, script)| {
                    script.contains(&format!("id = {tree_id}")) && script.contains(root_focus)
                })
            {
                initial_tree_ids.insert(tree_id.to_string());
                ambiguous_tags.retain(|owner| owner != tag);
            }
        }
        if !ambiguous_tags.is_empty() {
            diagnostics.push(format!(
                "{} country focus trees could not be identified as the starting tree (including {}); they are excluded from the opening route.",
                ambiguous_tags.len(),
                ambiguous_tags.iter().take(6).cloned().collect::<Vec<_>>().join(", ")
            ));
        }
        let tree_is_initial = trees
            .iter()
            .map(|tree| (tree.id.clone(), initial_tree_ids.contains(&tree.id)))
            .collect::<HashMap<_, _>>();
        let tree_tags = trees
            .iter()
            .map(|tree| (tree.id.as_str(), &tree.tags))
            .collect::<HashMap<_, _>>();
        let mut shared_memberships: HashMap<String, Vec<(String, bool)>> = HashMap::new();
        for tree in &trees {
            let is_initial = tree_is_initial.get(&tree.id).copied().unwrap_or(false);
            for shared_id in &tree.shared_focuses {
                shared_memberships
                    .entry(shared_id.clone())
                    .or_default()
                    .push((tree.id.clone(), is_initial));
            }
        }
        let shared_requirements = scripts
            .iter()
            .flat_map(|(_, script)| {
                blocks(script, "shared_focus = {").into_iter().map(|focus| {
                    let parents = blocks(&focus, "prerequisite = {")
                        .iter()
                        .flat_map(|group| script_values(group, "focus"))
                        .collect::<Vec<_>>();
                    (top_value(&focus, "id"), parents)
                })
            })
            .collect::<Vec<_>>();
        loop {
            let mut changed = false;
            for (id, parents) in &shared_requirements {
                let inherited = parents
                    .iter()
                    .filter_map(|id| shared_memberships.get(id))
                    .flatten()
                    .cloned()
                    .collect::<Vec<_>>();
                if inherited.is_empty() {
                    continue;
                }
                let memberships = shared_memberships.entry(id.clone()).or_default();
                for membership in inherited {
                    if !memberships.contains(&membership) {
                        memberships.push(membership);
                        changed = true;
                    }
                }
            }
            if !changed {
                break;
            }
        }

        for (source, script) in &scripts {
            for tree in blocks(script, "focus_tree = {") {
                let focus_tree_id = top_value(&tree, "id");
                let initial_tree = tree_is_initial
                    .get(&focus_tree_id)
                    .copied()
                    .unwrap_or(false);
                for b in blocks(&tree, "focus = {") {
                    add_focus_chapter(
                        &b,
                        source,
                        &focus_tree_id,
                        initial_tree,
                        Vec::new(),
                        &start_state,
                        &loc,
                        &icons,
                        &mut chapters,
                        &mut edges,
                        &supers,
                        &effects,
                    );
                }
            }
            for shared in blocks(script, "shared_focus = {") {
                let id = top_value(&shared, "id");
                let Some(memberships) = shared_memberships.get(&id) else {
                    continue;
                };
                for (focus_tree_id, initial_tree) in memberships {
                    add_focus_chapter(
                        &shared,
                        source,
                        focus_tree_id,
                        *initial_tree,
                        tree_tags
                            .get(focus_tree_id.as_str())
                            .map(|tags| (*tags).clone())
                            .unwrap_or_default(),
                        &start_state,
                        &loc,
                        &icons,
                        &mut chapters,
                        &mut edges,
                        &supers,
                        &effects,
                    );
                }
            }
        }
    } else {
        diagnostics.push(format!(
            "National focus scripts not found: {}",
            dir.display()
        ))
    }
    let mut unique_chapters = BTreeMap::new();
    for chapter in chapters {
        match unique_chapters.entry((chapter.focus_tree_id.clone(), chapter.id.clone())) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(chapter);
            }
            std::collections::btree_map::Entry::Occupied(mut entry) => {
                if chapter.initial_tree && !entry.get().initial_tree {
                    entry.insert(chapter);
                }
            }
        }
    }
    let mut chapters = unique_chapters.into_values().collect::<Vec<_>>();
    chapters.sort_by(|a, b| a.source.cmp(&b.source).then_with(|| a.id.cmp(&b.id)));

    let mut timeline = Vec::new();
    let events_dir = root.join("events");
    if mod_layers::directory_exists(&root, Path::new("events")) {
        for entry in mod_layers::entries(&root, "events")
            .filter(|entry| {
                entry.file_type().is_file()
                    && entry
                        .path()
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("txt"))
            })
        {
            let Ok(script) = decode(entry.path()) else {
                continue;
            };
            let country_tag = if entry
                .path()
                .file_stem()
                .is_some_and(|stem| stem.to_string_lossy().to_ascii_uppercase().contains("ZZZ"))
            {
                String::new()
            } else {
                source_country(entry.path(), &tags)
            };
            for (kind, marker) in [
                ("event", "country_event = {"),
                ("news", "news_event = {"),
                ("event", "unit_leader_event = {"),
            ] {
                for block in blocks(&script, marker) {
                    let id = top_value(&block, "id");
                    if id.is_empty() || is_shared_development_notice(entry.path(), &id) {
                        continue;
                    }
                    let entries = direct_statements(block_body(&block).unwrap_or_default())
                        .collect::<Vec<_>>();
                    let hidden = top_value(&block, "hidden") == "yes";
                    if !hidden && !entries.iter().any(|(key, _)| *key == "title") {
                        continue;
                    }
                    let (title, title_variants) = event_text(&entries, "title", &loc);
                    let (description, description_variants) = event_text(&entries, "desc", &loc);
                    let picture_key = top_value(&block, "picture");
                    let event_art = event_images.get(&icon_name(&picture_key).to_lowercase());
                    let (mut date, mut timing) = timing_label(&block);
                    let focus_progress = focus_progress_trigger(&block).filter(|trigger| {
                        chapters.iter().any(|chapter| chapter.id == trigger.focus)
                    });
                    if let Some(trigger) = &focus_progress {
                        edges.push((
                            format!("focus-progress:{}", trigger.focus),
                            id.clone(),
                            String::new(),
                        ));
                        timing = format!(
                            "国策 {} 的进度{} {}% 时触发",
                            trigger.focus,
                            if trigger.inclusive {
                                "达到"
                            } else {
                                "超过"
                            },
                            trigger.progress * 100.0
                        );
                    }
                    let autonomous = autonomous_date(&block);
                    let autonomous_trigger = if autonomous.is_none() {
                        autonomous_trigger(&block, &country_tag)
                    } else {
                        None
                    };
                    let manual_trigger = if autonomous.is_none()
                        && autonomous_trigger.is_none()
                        && focus_progress.is_none()
                    {
                        manual_trigger(&block, &country_tag)
                            .filter(|trigger| tags.contains(&trigger.owner))
                    } else {
                        None
                    };
                    if let Some(after) = autonomous
                        .as_ref()
                        .and_then(|schedule| schedule.after.as_ref())
                    {
                        date = Some(after.clone());
                    }
                    let immediate = blocks(&block, "immediate = {")
                        .into_iter()
                        .map(|effect| effects.expand_in_scope(&effect, 8, &mut 32))
                        .collect::<Vec<_>>();
                    let mut next_ids = Vec::new();
                    for effect in &immediate {
                        for (target, delay) in script_links(&effect) {
                            next_ids.push(target.clone());
                            edges.push((format!("{kind}:{id}"), target, delay));
                        }
                        for (target, relation) in decision_links(&effect) {
                            next_ids.push(target.clone());
                            edges.push((
                                format!("{kind}:{id} ({relation})"),
                                target,
                                String::new(),
                            ));
                        }
                    }
                    let mut options = Vec::new();
                    for option in blocks(&block, "option = {") {
                        let option_id = top_value(&option, "name");
                        if option_id.is_empty() {
                            continue;
                        }
                        let effect_preview = if hidden { reader_notes::EffectPreview::default() } else {
                            reader_notes::option_effects(&option, &loc, &tooltip_templates, &effects)
                        };
                        let mut option = effects.expand_in_scope(&option, 8, &mut 32);
                        // A tooltip describes effects but does not execute them.
                        for tooltip in blocks(&option, "effect_tooltip = {") {
                            if let Some(start) = option.find(&tooltip) {
                                option.replace_range(start..start + tooltip.len(), "");
                            }
                        }
                        let option_links = script_links(&option);
                        let mut option_next = option_links
                            .iter()
                            .map(|(target, _)| target.clone())
                            .collect::<Vec<_>>();
                        for (target, delay) in option_links {
                            edges.push((
                                format!("{kind}:{id}/option:{option_id}"),
                                target.clone(),
                                delay,
                            ));
                            next_ids.push(target);
                        }
                        for (target, relation) in decision_links(&option) {
                            edges.push((
                                format!("{kind}:{id}/option:{option_id} ({relation})"),
                                target.clone(),
                                String::new(),
                            ));
                            option_next.push(target.clone());
                            next_ids.push(target);
                        }
                        let route_effects = [
                            "set_country_flag",
                            "clr_country_flag",
                            "set_global_flag",
                            "clr_global_flag",
                            "load_focus_tree",
                        ]
                        .into_iter()
                        .flat_map(|command| {
                            script_values(&option, command)
                                .into_iter()
                                .map(move |value| format!("{command}:{value}"))
                        })
                        .collect();
                        options.push(StoryOption {
                            effect_preview,
                            music: supers.music_effects(&option),
                            mission_time_changes: mission_time_changes(&option),
                            trigger_checks: trigger_checks(&option),
                            trigger_condition: trigger_condition(&option),
                            conditional_routes: conditional_routes(&option, &country_tag),
                            mission_activations: mission_activations(&option),
                            credits: ending_credits(&option, &loc),
                            foreign_events: scoped_event_calls(&option, &country_tag),
                            event_calls: local_event_calls(&option, &country_tag),
                            flag_changes: country_flag_changes(&option),
                            focus_tree_changes: focus_tree_changes(&option),
                            route_effects,
                            target_tags: option_target_tags(&option),
                            focus_completions: block_focus_completions(&option),
                            super_event: supers
                                .resolve([option.as_str()], &loc)
                                .map(|scene| super_events::with_event_art(scene, event_art)),
                            id: option_id.clone(),
                            text: localized(&loc, &option_id),
                            next_ids: option_next,
                        });
                    }
                    next_ids.sort();
                    next_ids.dedup();
                    let scene = (!matches!(id.as_str(), "news.139" | "news.357" | "china.7"))
                        .then(|| {
                            supers
                                .resolve(immediate.iter().map(String::as_str), &loc)
                                .map(|scene| super_events::with_event_art(scene, event_art))
                        })
                        .flatten();
                    if matches!(id.as_str(), "news.139" | "news.357" | "china.7") {
                        for option in &mut options {
                            option.super_event = None;
                        }
                    }
                    timeline.push(TimelineItem {
                        decision_end_music: None,
                        music: immediate
                            .iter()
                            .flat_map(|effect| supers.music_effects(effect))
                            .collect(),
                        manual_trigger,
                        periodic_triggers: Vec::new(),
                        autonomous_trigger,
                        focus_progress_trigger: focus_progress,
                        automatic_timeout: false,
                        hidden,
                        verified_startup: Vec::new(),
                        startup_trigger: startup_trigger(&block),
                        decision_guarded_calls: Vec::new(),
                        never_manually_available: false,
                        mission_time_changes: immediate
                            .iter()
                            .flat_map(|effect| mission_time_changes(effect))
                            .collect(),
                        trigger_checks: trigger_checks(&block),
                        conditional_routes: immediate
                            .iter()
                            .flat_map(|effect| conditional_routes(effect, &country_tag))
                            .collect(),
                        mission_activations: immediate
                            .iter()
                            .flat_map(|effect| mission_activations(effect))
                            .collect(),
                        immediate_calls: immediate
                            .iter()
                            .flat_map(|effect| local_event_calls(effect, &country_tag))
                            .collect(),
                        mission: None,
                        decision_checks: Vec::new(),
                        decision_condition: None,
                        decision_focuses: Vec::new(),
                        decision_event_calls: Vec::new(),
                        credits: immediate
                            .iter()
                            .flat_map(|effect| ending_credits(effect, &loc))
                            .collect(),
                        broadcast: kind == "news"
                            && top_value(&block, "major") == "yes"
                            && blocks(&block, "major_trigger = {").is_empty(),
                        broadcast_calls: immediate
                            .iter()
                            .flat_map(|effect| direct_event_calls(effect, true))
                            .collect(),
                        foreign_events: immediate
                            .iter()
                            .flat_map(|effect| scoped_event_calls(effect, &country_tag))
                            .collect(),
                        focus_tree_changes: immediate
                            .iter()
                            .flat_map(|effect| focus_tree_changes(effect))
                            .collect(),
                        flag_changes: immediate
                            .iter()
                            .flat_map(|effect| country_flag_changes(effect))
                            .collect(),
                        focus_completions: immediate
                            .iter()
                            .flat_map(|effect| block_focus_completions(effect))
                            .collect(),
                        super_event: scene,
                        kind: kind.into(),
                        country_tag: country_tag.clone(),
                        title: if title.is_empty() {
                            localized(&loc, &format!("{id}.t"))
                        } else {
                            title
                        },
                        description,
                        id,
                        title_variants,
                        description_variants,
                        date,
                        date_after: autonomous.is_some(),
                        date_before: autonomous
                            .as_ref()
                            .and_then(|schedule| schedule.before.clone()),
                        date_country: autonomous
                            .as_ref()
                            .and_then(|schedule| schedule.tag.clone()),
                        date_focuses: autonomous
                            .map(|schedule| schedule.focuses)
                            .unwrap_or_default(),
                        timing,
                        source: entry.path().display().to_string(),
                        image_path: event_images
                            .get(&icon_name(&picture_key).to_lowercase())
                            .map(|path| path.display().to_string()),
                        start_day: None,
                        start_timing: None,
                        next_ids,
                        linked_from: Vec::new(),
                        options,
                    });
                }
            }
        }
    } else {
        diagnostics.push(format!("Event scripts not found: {}", events_dir.display()));
    }

    let decisions_dir = root.join("common/decisions");
    if mod_layers::directory_exists(&root, Path::new("common/decisions")) {
        for entry in mod_layers::entries(&root, "common/decisions")
            .filter(|entry| {
                entry.file_type().is_file()
                    && entry
                        .path()
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("txt"))
            })
        {
            if entry
                .file_name()
                .to_string_lossy()
                .to_ascii_lowercase()
                .contains("debug")
            {
                continue;
            }
            let Ok(script) = decode(entry.path()) else {
                continue;
            };
            let file_tag = source_country(entry.path(), &tags);
            for (category, category_block) in named_blocks(&script) {
                if category.to_ascii_lowercase().contains("debug") {
                    continue;
                }
                let Some(open) = category_block.find('{') else {
                    continue;
                };
                let Some(close) = category_block.rfind('}') else {
                    continue;
                };
                let category_tag = source_country(Path::new(&category), &tags);
                let country_tag = if category_tag.is_empty() {
                    file_tag.clone()
                } else {
                    category_tag
                };
                for (id, block) in named_blocks(&category_block[open + 1..close]) {
                    if id.to_ascii_lowercase().contains("_debug") {
                        continue;
                    }
                    let explicit_owner = direct_statements(block_body(&block).unwrap_or_default())
                        .find_map(|(key, body)| {
                            (key == "allowed").then(|| block_body(body)).flatten()
                        })
                        .and_then(|body| {
                            let owners = direct_statements(body)
                                .filter(|(key, value)| {
                                    matches!(*key, "tag" | "original_tag") && tags.contains(*value)
                                })
                                .map(|(_, value)| value)
                                .collect::<std::collections::BTreeSet<_>>();
                            (owners.len() == 1)
                                .then(|| owners.into_iter().next())
                                .flatten()
                        });
                    let country_tag = explicit_owner
                        .map(str::to_string)
                        .unwrap_or_else(|| country_tag.clone());
                    let timeout = top_value(&block, "days_mission_timeout");
                    let is_mission = !timeout.is_empty();
                    if timeout.is_empty()
                        && top_value(&block, "cost").is_empty()
                        && !direct_statements(block_body(&block).unwrap_or_default())
                            .any(|(key, _)| matches!(key, "complete_effect" | "remove_effect"))
                    {
                        continue;
                    }
                    let kind = if is_mission { "mission" } else { "decision" };
                    let timing = if is_mission {
                        "限时任务；脚本未给出实际触发日期".to_string()
                    } else {
                        "满足可用条件后由玩家选择；日期不固定".to_string()
                    };
                    let mut decision = block.clone();
                    for effect in ["complete_effect", "remove_effect"] {
                        for original in blocks(&block, &format!("{effect} = {{")) {
                            let expanded = effects.expand_in_scope(&original, 8, &mut 32);
                            if expanded != original {
                                decision = decision.replacen(&original, &expanded, 1);
                            }
                        }
                    }
                    let mut next_ids = Vec::new();
                    for effect_name in [
                        "complete_effect",
                        "timeout_effect",
                        "remove_effect",
                        "cancel_effect",
                    ] {
                        for effect in blocks(&decision, &format!("{effect_name} = {{")) {
                            for (target, delay) in script_links(&effect) {
                                next_ids.push(target.clone());
                                edges.push((format!("{kind}:{id}"), target, delay));
                            }
                        }
                    }
                    let mission = timeout.parse().ok().map(|days| Mission {
                        days,
                        shared_resolution: shared_war_resolution(&id),
                        complete: mission_effect(
                            &block,
                            "complete_effect",
                            &country_tag,
                            &effects,
                            &supers,
                        ),
                        timeout: mission_effect(
                            &block,
                            "timeout_effect",
                            &country_tag,
                            &effects,
                            &supers,
                        ),
                        cancel: mission_effect(
                            &block,
                            "cancel_effect",
                            &country_tag,
                            &effects,
                            &supers,
                        ),
                    });
                    if let Some(mission) = &mission {
                        for effect in [&mission.complete, &mission.timeout, &mission.cancel] {
                            for (target, delay) in effect
                                .calls
                                .iter()
                                .cloned()
                                .chain(
                                    effect
                                        .foreign_events
                                        .iter()
                                        .map(|link| (link.target.clone(), link.delay.clone())),
                                )
                                .chain(conditional_event_links(&effect.conditional_routes))
                            {
                                edges.push((format!("{kind}:{id}"), target.clone(), delay));
                                next_ids.push(target);
                            }
                        }
                    }
                    next_ids.sort();
                    next_ids.dedup();
                    let (date, _) = timing_label(&block);
                    let gates = direct_statements(block_body(&block).unwrap_or_default())
                        .filter(|(key, _)| matches!(*key, "visible" | "available"))
                        .filter_map(|(_, value)| block_body(value))
                        .collect::<Vec<_>>();
                    let decision_condition = ScriptCondition::All(
                        direct_statements(block_body(&block).unwrap_or_default())
                            .filter(|(key, _)| matches!(*key, "allowed" | "visible" | "available"))
                            .filter_map(|(_, value)| block_body(value))
                            .map(|body| script_condition_in_country(body, 8, &country_tag))
                            .collect(),
                    );
                    let automatic_timeout = mission.as_ref().is_some_and(|mission| {
                        matches!(
                            id.as_str(),
                            "NATO_intervention_timer" | "PRC_korean_war_joining"
                        ) && mission.complete.calls.is_empty()
                            && mission.complete.foreign_events.is_empty()
                            && mission.cancel.calls.is_empty()
                            && mission.cancel.foreign_events.is_empty()
                            && !mission.timeout.calls.is_empty()
                    });
                    timeline.push(TimelineItem {
                        decision_end_music: (!is_mission)
                            .then(|| {
                                let days =
                                    top_value(&decision, "days_remove").parse::<u32>().ok()?;
                                let effects = blocks(&decision, "remove_effect = {");
                                let music = effects
                                    .iter()
                                    .flat_map(|effect| supers.music_effects(effect))
                                    .collect::<Vec<_>>();
                                (!music.is_empty()).then(|| DecisionEndMusic {
                                    days,
                                    effects: music,
                                    flags: effects
                                        .iter()
                                        .flat_map(|effect| country_flag_changes(effect))
                                        .collect(),
                                    conditional_routes: effects
                                        .iter()
                                        .flat_map(|effect| conditional_routes(effect, &country_tag))
                                        .collect(),
                                })
                            })
                            .flatten(),
                        music: if is_mission {
                            Vec::new()
                        } else {
                            blocks(&decision, "complete_effect = {")
                                .iter()
                                .flat_map(|effect| supers.decision_music(effect))
                                .collect()
                        },
                        manual_trigger: None,
                        periodic_triggers: Vec::new(),
                        autonomous_trigger: None,
                        focus_progress_trigger: None,
                        automatic_timeout,
                        hidden: false,
                        verified_startup: Vec::new(),
                        startup_trigger: None,
                        decision_guarded_calls: decision_guarded_calls(&decision),
                        never_manually_available: gates
                            .iter()
                            .any(|body| never_manually_available(body)),
                        mission_time_changes: blocks(&block, "complete_effect = {")
                            .iter()
                            .flat_map(|effect| mission_time_changes(effect))
                            .collect(),
                        trigger_checks: None,
                        conditional_routes: blocks(&decision, "complete_effect = {")
                            .iter()
                            .flat_map(|effect| conditional_routes(effect, &country_tag))
                            .collect(),
                        immediate_calls: Vec::new(),
                        mission_activations: blocks(&block, "complete_effect = {")
                            .iter()
                            .flat_map(|effect| mission_activations(effect))
                            .collect(),
                        mission,
                        decision_checks: gates
                            .iter()
                            .filter_map(|body| focus_flag_checks(body, false))
                            .flatten()
                            .collect(),
                        decision_condition: (!is_mission).then_some(decision_condition),
                        decision_focuses: gates
                            .iter()
                            .flat_map(|body| direct_statements(body))
                            .filter(|(key, value)| {
                                *key == "has_completed_focus" && !value.starts_with('{')
                            })
                            .map(|(_, value)| value.trim_matches('"').to_string())
                            .collect(),
                        decision_event_calls: decision_event_calls(&decision, &country_tag),
                        credits: blocks(&block, "complete_effect = {")
                            .iter()
                            .flat_map(|effect| ending_credits(effect, &loc))
                            .collect(),
                        title_variants: Vec::new(),
                        description_variants: Vec::new(),
                        broadcast: false,
                        broadcast_calls: Vec::new(),
                        foreign_events: blocks(&decision, "complete_effect = {")
                            .iter()
                            .flat_map(|effect| scoped_event_calls(effect, &country_tag))
                            .collect(),
                        focus_tree_changes: blocks(&block, "complete_effect = {")
                            .iter()
                            .flat_map(|effect| focus_tree_changes(effect))
                            .collect(),
                        flag_changes: blocks(&block, "complete_effect = {")
                            .iter()
                            .flat_map(|effect| country_flag_changes(effect))
                            .collect(),
                        focus_completions: blocks(&decision, "complete_effect = {")
                            .iter()
                            .flat_map(|effect| block_focus_completions(effect))
                            .collect(),
                        super_event: supers.resolve(
                            blocks(&block, "complete_effect = {")
                                .iter()
                                .map(String::as_str),
                            &loc,
                        ),
                        id: id.clone(),
                        kind: kind.into(),
                        country_tag: country_tag.clone(),
                        title: localized(&loc, &id),
                        description: localized(&loc, &format!("{id}_desc")),
                        date,
                        date_after: false,
                        date_before: None,
                        date_focuses: Vec::new(),
                        date_country: None,
                        timing,
                        source: entry.path().display().to_string(),
                        image_path: None,
                        start_day: None,
                        start_timing: None,
                        next_ids,
                        linked_from: Vec::new(),
                        options: Vec::new(),
                    });
                }
            }
        }
    } else {
        diagnostics.push(format!(
            "Decision scripts not found: {}",
            decisions_dir.display()
        ));
    }

    let mut verified_startups = HashMap::<String, Vec<ForeignEvent>>::new();
    let mut periodic = HashMap::<String, Vec<PeriodicTrigger>>::new();
    let mut election_dates = HashMap::new();
    for entry in mod_layers::entries(&root, "history/countries")
        .filter(|entry| {
            entry.file_type().is_file()
                && entry
                    .path()
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("txt"))
        })
    {
        let recipient = source_country(entry.path(), &tags);
        if recipient.is_empty() {
            continue;
        }
        let Ok(script) = decode(entry.path()) else {
            continue;
        };
        if let Some(date) = next_election_date(&script) {
            election_dates.insert(recipient.clone(), date);
        }
        // Undated history executes when the bookmark loads. Dated blocks and
        // conditional/DLC branches need their own evaluation.
        let mut calls = Vec::new();
        collect_startup_calls(
            &format!("history = {{ {script}\n }}"),
            &tags,
            Some(&recipient),
            8,
            &mut calls,
        );
        for link in calls {
            edges.push((
                format!("startup:{}", link.recipient),
                link.target.clone(),
                link.delay.clone(),
            ));
            verified_startups
                .entry(link.target.to_ascii_lowercase())
                .or_default()
                .push(link);
        }
    }
    let on_actions_dir = root.join("common/on_actions");
    let mut war_news_missions = HashMap::<String, Vec<String>>::new();
    const SECOND_WAR_NEWS: [&str; 7] = [
        "news.291",
        "news.538",
        "news.537",
        "news.409",
        "news.1929",
        "news.411",
        "news.576",
    ];
    for chapter in &chapters {
        for mission in chapter
            .mission_activations
            .iter()
            .filter(|id| shared_war_resolution(id))
        {
            for (news, _) in &chapter.broadcast_calls {
                if mission == "second_nato_war_mission" && !SECOND_WAR_NEWS.contains(&news.as_str())
                {
                    continue;
                }
                let missions = war_news_missions
                    .entry(news.to_ascii_lowercase())
                    .or_default();
                if !missions.contains(mission) {
                    missions.push(mission.clone());
                }
            }
        }
    }
    if mod_layers::directory_exists(&root, Path::new("common/on_actions")) {
        for entry in mod_layers::entries(&root, "common/on_actions")
            .filter(|entry| {
                entry.file_type().is_file()
                    && entry
                        .path()
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("txt"))
            })
        {
            let Ok(script) = decode(entry.path()) else {
                continue;
            };
            for hook in blocks(&script, "on_new_term_election = {") {
                let weighted = blocks(&hook, "random_events = {");
                let Some(list) = weighted.first().filter(|_| weighted.len() == 1) else {
                    continue;
                };
                let entries =
                    direct_statements(block_body(list).unwrap_or_default()).collect::<Vec<_>>();
                let [(weight, target)] = entries.as_slice() else {
                    continue;
                };
                if !weight.parse::<u32>().is_ok_and(|weight| weight > 0) {
                    continue;
                }
                for item in timeline
                    .iter_mut()
                    .filter(|item| item.kind == "event" && !item.hidden && item.id == *target)
                {
                    let Some(date) = election_dates.get(&item.country_tag) else {
                        continue;
                    };
                    if item.date.is_some() || !edges.iter().all(|(_, id, _)| id != &item.id) {
                        continue;
                    }
                    item.date = Some(date.clone());
                    item.date_country = Some(item.country_tag.clone());
                    item.timing = "由选举周期和开局历史日期确定".into();
                    edges.push((
                        "on-action:on_new_term_election".into(),
                        item.id.clone(),
                        String::new(),
                    ));
                }
            }
            if script.contains("third_euro_war_mission") {
                for hook in blocks(&script, "on_declare_war = {") {
                    for effect in blocks(&hook, "effect = {") {
                        for (_, branch) in effect_commands(&effect)
                            .into_iter()
                            .filter(|(key, _)| *key == "if")
                        {
                            if !mission_activations(branch)
                                .iter()
                                .any(|id| id == "third_euro_war_mission")
                            {
                                continue;
                            }
                            for news in blocks(branch, "news_event = {")
                                .iter()
                                .map(|call| top_value(call, "id"))
                                .chain(script_values(branch, "news_event"))
                                .filter(|id| id.contains('.'))
                            {
                                war_news_missions.insert(
                                    news.to_ascii_lowercase(),
                                    vec!["third_euro_war_mission".into()],
                                );
                            }
                        }
                    }
                }
            }
            for (hook, block) in blocks(&script, "on_actions = {")
                .iter()
                .flat_map(|actions| named_blocks(block_body(actions).unwrap_or_default()))
            {
                let Some((frequency, owner)) = hook.rsplit_once('_') else {
                    continue;
                };
                let interval = match frequency {
                    "on_daily" => 1,
                    "on_weekly" => 7,
                    _ => continue,
                };
                if !tags.contains(owner) {
                    continue;
                }
                for (key, effect) in direct_statements(block_body(&block).unwrap_or_default()) {
                    if key != "effect" {
                        continue;
                    }
                    collect_periodic_calls(
                        effect,
                        owner,
                        owner,
                        interval,
                        &tags,
                        &[],
                        8,
                        &mut periodic,
                    );
                }
            }
            for startup in blocks(&script, "on_startup = {") {
                for effect in blocks(&startup, "effect = {") {
                    let effect = effects.expand_in_scope(&effect, 8, &mut 64);
                    let mut calls = Vec::new();
                    collect_startup_calls(&effect, &tags, None, 8, &mut calls);
                    for link in calls {
                        edges.push((
                            format!("startup:{}", link.recipient),
                            link.target.clone(),
                            link.delay.clone(),
                        ));
                        verified_startups
                            .entry(link.target.to_ascii_lowercase())
                            .or_default()
                            .push(link);
                    }
                    let Some(open) = effect.find('{') else {
                        continue;
                    };
                    let Some(close) = effect.rfind('}') else {
                        continue;
                    };
                    for (scope_name, scope) in named_blocks(&effect[open + 1..close]) {
                        if tags.contains(&scope_name) {
                            for (target, delay) in script_links(&scope) {
                                edges.push((format!("startup:{scope_name}"), target, delay));
                            }
                        } else if scope_name == "every_country" {
                            for condition in blocks(&scope, "if = {") {
                                let guarded_tags = blocks(&condition, "limit = {")
                                    .into_iter()
                                    .flat_map(|limit| script_values(&limit, "tag"))
                                    .collect::<std::collections::BTreeSet<_>>();
                                for tag in &guarded_tags {
                                    if !tags.contains(tag) {
                                        continue;
                                    }
                                    for (target, delay) in script_links(&condition) {
                                        edges.push((format!("startup:{tag}"), target, delay));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        diagnostics.push(format!(
            "Opening-event scripts not found: {}",
            on_actions_dir.display()
        ));
    }

    let mut timeline_indices: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, item) in timeline.iter().enumerate() {
        timeline_indices
            .entry(item.id.to_ascii_lowercase())
            .or_default()
            .push(index);
    }
    for (source, target, delay) in edges {
        let Some(indices) = timeline_indices.get(&target.to_ascii_lowercase()) else {
            continue;
        };
        for index in indices {
            let item = &mut timeline[*index];
            if let Some(tag) = source.strip_prefix("startup:") {
                let days = delay.split(',').find_map(|part| {
                    part.trim()
                        .strip_prefix("days ")
                        .and_then(|value| value.parse::<i32>().ok())
                });
                let hours = delay.split(',').find_map(|part| {
                    part.trim()
                        .strip_prefix("hours ")
                        .and_then(|value| value.parse::<i32>().ok())
                });
                let random_delay = delay.contains("random_");
                let day = if random_delay {
                    None
                } else {
                    Some(days.unwrap_or(0) + hours.unwrap_or(0) / 24)
                };
                let replace_schedule = match (item.start_day, day) {
                    (_, None) => item.start_day.is_none(),
                    (Some(current), Some(candidate)) => candidate < current,
                    (None, Some(_)) => true,
                };
                if replace_schedule {
                    item.start_day = day;
                    item.start_timing = Some(if delay.is_empty() {
                        format!("{tag} 开局触发")
                    } else if random_delay {
                        format!("{tag} 开局后随机延迟（{delay}）")
                    } else {
                        format!("{tag} 开局后 {delay} 触发")
                    });
                }
            }
            let relation = if delay.is_empty() {
                source.clone()
            } else {
                format!("{source} ({delay})")
            };
            item.linked_from.push(relation);
        }
    }
    for item in &mut timeline {
        item.periodic_triggers = periodic
            .remove(&item.id.to_ascii_lowercase())
            .unwrap_or_default();
        item.verified_startup = verified_startups
            .get(&item.id.to_ascii_lowercase())
            .cloned()
            .unwrap_or_default();
        item.linked_from.sort();
        item.linked_from.dedup();
    }
    timeline.sort_by(|a, b| {
        a.country_tag
            .cmp(&b.country_tag)
            .then_with(|| a.date.cmp(&b.date))
            .then_with(|| a.source.cmp(&b.source))
            .then_with(|| a.id.cmp(&b.id))
    });

    let great_asian_war_start = effects.0.get("PRC_gaw_start").and_then(|effect| {
        effect_commands(effect)
            .into_iter()
            .filter(|(key, scope)| {
                *key == "JAP"
                    && mission_activations(scope)
                        .iter()
                        .any(|id| id == "JAP_GAW_war_exhaustion")
            })
            .flat_map(|(_, scope)| direct_event_calls(scope, false))
            .find(|(id, _)| id == "jap.1002")
            .map(|(target, delay)| ForeignEvent {
                recipient: "JAP".into(),
                target,
                delay,
            })
    });
    let great_asian_war_news = great_asian_war_start.as_ref().and_then(|_| {
        let effect = effects.0.get("PRC_gaw_start")?;
        direct_event_calls(effect, true)
            .into_iter()
            .find(|(id, _)| id == "news.415")
    });
    let great_asian_war_recipients = great_asian_war_start
        .as_ref()
        .and_then(|_| effects.0.get("PRC_gaw_start"))
        .map(|effect| scoped_event_calls(effect, "PRC"))
        .unwrap_or_default();
    let american_crisis_anchor = {
        let source = timeline
            .iter()
            .find(|item| item.id == "usa.103" && item.kind == "event");
        let outbreak = timeline
            .iter()
            .find(|item| item.id == "usa.104" && item.kind == "event");
        if let (Some(source), Some(outbreak)) = (source, outbreak) {
            if source.immediate_calls.iter().any(|(id, _)| id == "usa.104")
                && outbreak
                    .broadcast_calls
                    .iter()
                    .any(|(id, _)| id == "news.115")
                && outbreak
                    .flag_changes
                    .iter()
                    .any(|flag| flag.flag == "global:USA_american_civil_war_global" && flag.present)
            {
                Some(source.id.clone())
            } else {
                None
            }
        } else {
            None
        }
    };
    if american_crisis_anchor.is_some() {
        if let Some(news) = timeline
            .iter_mut()
            .find(|item| item.id == "news.115" && item.broadcast)
        {
            news.flag_changes.push(FlagChange {
                flag: "global:USA_american_civil_war_global".into(),
                present: true,
            });
        }
    }
    let mut unlinked = timeline
        .iter()
        .filter(|item| {
            !item.hidden
                && item.kind == "event"
                && item.linked_from.is_empty()
                && item.date.is_none()
                && !item.date_after
                && item.autonomous_trigger.is_none()
                && item.manual_trigger.is_none()
        })
        .count();
    let mut unlinked_news = timeline
        .iter()
        .filter(|item| {
            !item.hidden
                && item.kind == "news"
                && item.linked_from.is_empty()
                && item.date.is_none()
                && !item.date_after
                && item.autonomous_trigger.is_none()
                && item.manual_trigger.is_none()
        })
        .count();
    // Index reactive hooks as source evidence. They do not supply a date:
    // e.g. a capitulation still needs an actual war result in the reader.
    let mut reactive_sources = HashMap::<String, std::collections::BTreeSet<String>>::new();
    let mut event_references = HashMap::new();
    let mut wartime_hooks = Vec::new();
    for directory in [
        "on_actions",
        "bop",
        "ideas",
        "technologies",
        "operations",
        "special_projects",
        "military_industrial_organization",
        "resistance_compliance_modifiers",
        "scripted_guis",
    ] {
        for entry in mod_layers::entries(&root, Path::new("common").join(directory))
            .filter(|entry| {
                entry.file_type().is_file()
                    && entry
                        .path()
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("txt"))
            })
        {
            let Ok(script) = decode(entry.path()) else {
                continue;
            };
            let file = entry.file_name().to_string_lossy();
            if file.to_ascii_lowercase().contains("debug") {
                continue;
            }
            let blocks = if directory == "on_actions" {
                blocks(&script, "on_actions = {")
                    .iter()
                    .flat_map(|block| named_blocks(block_body(block).unwrap_or_default()))
                    .filter(|(key, _)| key != "on_startup")
                    .collect()
            } else {
                named_blocks(&script)
            };
            for (key, block) in blocks {
                if directory == "on_actions" {
                    index_wartime_hook(
                        &key,
                        &block,
                        &format!("common/on_actions/{file}"),
                        &effects,
                        &mut wartime_hooks,
                    );
                }
                for id in effects.referenced_events(&block, &mut event_references) {
                    reactive_sources
                        .entry(id.to_ascii_lowercase())
                        .or_default()
                        .insert(format!("common/{directory}/{file} · {key}"));
                }
            }
        }
    }
    let wartime_targets = wartime_hooks
        .iter()
        .flat_map(|hook| hook.arms.iter())
        .flat_map(|arm| arm.calls.iter())
        .map(|call| call.target.to_ascii_lowercase())
        .collect::<std::collections::BTreeSet<_>>();
    let mut reactive_only = 0;
    let mut wartime_events = 0;
    let mut wartime_news = 0;
    for item in &mut timeline {
        let Some(sources) = reactive_sources.get(&item.id.to_ascii_lowercase()) else {
            continue;
        };
        if item.hidden
            || !item.linked_from.is_empty()
            || item.date.is_some()
            || item.date_after
            || item.manual_trigger.is_some()
        {
            continue;
        }
        if item.kind == "event" && item.autonomous_trigger.is_none() {
            unlinked -= 1;
            if wartime_targets.contains(&item.id.to_ascii_lowercase()) {
                wartime_events += 1;
            } else {
                reactive_only += 1;
            }
        } else if item.kind == "news" && item.autonomous_trigger.is_none() {
            unlinked_news -= 1;
            if item.broadcast && wartime_targets.contains(&item.id.to_ascii_lowercase()) {
                wartime_news += 1;
            }
        }
        item.timing = format!(
            "由游戏状态或界面操作触发 · {}",
            sources.iter().cloned().collect::<Vec<_>>().join("、")
        );
    }
    if reactive_only > 0 {
        diagnostics.push(format!("{reactive_only} visible country events have a reactive script source but no calendar date."));
    }
    if wartime_events > 0 {
        diagnostics.push(format!("{wartime_events} visible country events can be offered at an identified wartime checkpoint."));
    }
    if wartime_news > 0 {
        diagnostics.push(format!("{wartime_news} visible international news events can be offered at an identified wartime checkpoint."));
    }
    let periodic_candidates = timeline
        .iter()
        .filter(|item| !item.hidden && !item.periodic_triggers.is_empty())
        .count();
    if periodic_candidates > 0 {
        diagnostics.push(format!(
            "{periodic_candidates} visible events have country-specific periodic script calls."
        ));
    }
    if unlinked > 0 {
        diagnostics.push(format!(
            "{unlinked} visible country events have no indexed date or upstream link."
        ));
    }
    if unlinked_news > 0 {
        diagnostics.push(format!(
            "{unlinked_news} visible news events have no indexed date or upstream link."
        ));
    }
    ScanResult {
        inline_assets: reader_notes::inline_assets(&root),
        text_colors: reader_notes::text_colors(&root, game),
        opening_notes,
        wartime_hooks,
        war_news_missions,
        american_crisis_anchor,
        great_asian_war_start,
        great_asian_war_recipients,
        great_asian_war_news,
        default_game_rules: default_game_rules(&root),
        start_date,
        initial_country_flags: start_state.flags,
        country_names,
        chapters,
        timeline,
        diagnostics,
    }
}
#[tauri::command]
fn load_focus_image(mod_path: String, focus_id: String) -> Option<String> {
    let root = PathBuf::from(&mod_path);
    let icon = mod_layers::entries(&root, "common/national_focus")
        .filter(|e| {
            e.file_type().is_file()
                && e.path()
                    .extension()
                    .is_some_and(|x| x.eq_ignore_ascii_case("txt"))
        })
        .filter_map(|e| decode(e.path()).ok())
        .flat_map(|s| blocks(&s, "focus = {"))
        .find(|b| top_value(b, "id") == focus_id)
        .map(|b| top_value(&b, "icon"))?;
    let p = icon_path(&root, &icon)?;
    Some(format!(
        "data:image/png;base64,{}",
        STANDARD.encode(fs::read(p).ok()?)
    ))
}
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(index_cache::IndexStore::default());
    #[cfg(target_os = "android")]
    let builder = builder.plugin(tauri_plugin_fs::init());

    builder
        .invoke_handler(tauri::generate_handler![
            discover,
            #[cfg(target_os = "android")]
            android_library::import_archive,
            #[cfg(target_os = "android")]
            android_library::download_archive,
            scan_workshop_dir,
            index_cache::scan_mod,
            index_cache::index_secondary_countries,
            reader_notes::load_inline_flag,
            super_events::load_super_event_image,
            super_events::load_tfr_menu_art,
            super_events::load_tfr_super_event_skin,
            super_events::load_tno_super_event_skin,
            load_focus_image
        ])
        .run(tauri::generate_context!())
        .expect("error while running HOI4Reader");
}

#[cfg(test)]
mod story_script_tests {
    use super::*;

    #[test]
    fn cached_script_references_keep_all_events_when_effects_call_each_other() {
        let effects = ScriptedEffects(HashMap::from([
            ("first".into(), "{ second = yes\n country_event = { id = test.1 } }".into()),
            ("second".into(), "{ first = yes\n news_event = { id = test.2 } }".into()),
        ]));
        let mut cache = HashMap::new();
        let expected = ["test.1".to_owned(), "test.2".to_owned()].into_iter().collect();
        assert_eq!(effects.referenced_events("{ first = yes }", &mut cache), expected);
        assert_eq!(effects.referenced_events("{ second = yes }", &mut cache), expected);
    }

    #[test]
    fn derives_the_regular_election_date_from_country_history() {
        let history = r#"add_stability = 0.1
            set_politics = { last_election = "2016.11.3" elections_allowed = yes election_frequency = 48 }"#;
        assert_eq!(next_election_date(history).as_deref(), Some("2020.11.3"));
        assert_eq!(
            next_election_date(&history.replace("allowed = yes", "allowed = no")),
            None
        );
    }

    #[test]
    fn preserves_cross_country_recipients_and_uppercase_else() {
        let block =
            "effect = { SOV = { GER = { country_event = { id = germany.1111 days = 3 } } } \
            FRA = { if = { limit = { has_country_flag = alt } country_event = france.2810 } \
            ELSE = { country_event = france.2812 } } }";
        let direct = scoped_event_calls(block, "SOV");
        assert_eq!(
            direct.as_slice(),
            &[ForeignEvent {
                recipient: "GER".into(),
                target: "germany.1111".into(),
                delay: "days 3".into()
            }]
        );
        let result = conditional_routes(block, "SOV");
        assert_eq!(result[0][1].calls, [("france.2812".into(), String::new())]);
    }

    #[test]
    fn indexes_only_wartime_hooks_with_a_required_war_and_preserves_delays() {
        let script = r#"on_state_control_changed = { effect = {
            if = { limit = { ROOT = { has_global_flag = SOV_nato_war_begins_global tag = SOV } }
                FRA = { country_event = { id = france.1 days = 5 } } }
            else_if = { limit = { FROM = { tag = SOV } } FRA = { country_event = france.2 } }
            if = { limit = { OR = { has_global_flag = SOV_nato_war_begins_global tag = SOV } } news_event = news.3 }
        } }"#;
        let mut hooks = Vec::new();
        index_wartime_hook(
            "on_state_control_changed",
            script,
            "test.txt",
            &ScriptedEffects::default(),
            &mut hooks,
        );
        assert_eq!(hooks.len(), 1);
        assert_eq!(hooks[0].arms.len(), 2);
        assert_eq!(hooks[0].arms[0].calls[0].recipient, "FRA");
        assert_eq!(hooks[0].arms[0].calls[0].delay, "days 5");
        assert_eq!(hooks[0].arms[1].calls[0].target, "france.2");
    }

    #[test]
    fn anchors_unknown_conditions_to_their_script_date_and_owner() {
        let script = "country_event = { fire_only_once = yes trigger = { tag = BEL BEL = { has_stability < 0.20 date > 2023.01.01 } } }";
        let trigger = manual_trigger(script, "FRA").unwrap();
        assert_eq!(trigger.owner, "BEL");
        assert_eq!(trigger.earliest.as_deref(), Some("2023.01.01"));
        assert!(!trigger.earliest_inclusive);
        assert_eq!(
            trigger.prompt.as_deref(),
            Some("此时国家稳定度是否低于 20%？")
        );
        assert!(manual_trigger(&script.replace("date > 2023.01.01", ""), "FRA").is_none());
        assert!(manual_trigger(
            &script.replace("date > 2023.01.01", "OR = { date > 2023.01.01 }"),
            "FRA"
        )
        .is_none());
    }

    #[test]
    fn keeps_conditional_news_in_the_country_scope_of_the_war_outcome() {
        let effect = mission_effect("mission = { cancel_effect = { GER = { if = { limit = { has_country_flag = left } news_event = test.1 } else = { news_event = test.2 } } } }",
            "cancel_effect", "CUB", &ScriptedEffects::default(), &super_events::Catalog::default());
        let chain = &effect.conditional_routes[0];
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0].owner.as_deref(), Some("GER"));
        assert_eq!(chain[1].calls[0].0, "test.2");
        assert!(effect.calls.is_empty());
        assert_eq!(
            mission_activations(
                "effect = { ZZZ = { activate_mission = second_nato_war_mission } }"
            ),
            ["second_nato_war_mission"]
        );
    }

    #[test]
    fn indexes_startup_calls_with_verified_recipients_only() {
        let tags = ["PRC".into(), "JAP".into(), "GER".into()]
            .into_iter()
            .collect();
        let script = "effect = {
            every_country = {
                limit = { OR = { tag = PRC original_tag = JAP } }
                hidden_effect = { country_event = { id = sample.1 days = 4 } }
            }
            GER = {
                if = { limit = { has_war = yes } country_event = sample.2 }
                country_event = sample.3
            }
        }";
        let mut calls = Vec::new();
        collect_startup_calls(script, &tags, None, 8, &mut calls);
        assert_eq!(
            calls
                .iter()
                .map(|link| (
                    link.recipient.as_str(),
                    link.target.as_str(),
                    link.delay.as_str()
                ))
                .collect::<Vec<_>>(),
            [
                ("JAP", "sample.1", "days 4"),
                ("PRC", "sample.1", "days 4"),
                ("GER", "sample.3", "")
            ]
        );
    }

    #[test]
    fn follows_named_domestic_leader_events_without_treating_foreign_leaders_as_local() {
        let decision = r#"decision = {
            complete_effect = {
                hidden_effect = {
                    PRC_general = { unit_leader_event = { id = china.183 days = 45 } }
                    JAP_general = { unit_leader_event = japan.1 }
                    effect_tooltip = { unit_leader_event = china.184 }
                }
            }
        }"#;
        assert_eq!(
            decision_event_calls(decision, "PRC"),
            [("china.183".into(), "days 45".into())]
        );
    }

    #[test]
    fn keeps_focus_guarded_event_calls_and_their_delays() {
        let routes = conditional_routes(
            "completion_reward = { hidden_effect = {
                if = { limit = { has_completed_focus = PRC_test_left }
                       country_event = { id = china.18 hours = 3 } }
                else_if = { limit = { has_completed_focus = PRC_test_liberal }
                            country_event = { id = china.19 hours = 3 } }
            } }",
            "PRC",
        );
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].len(), 2);
        assert_eq!(routes[0][0].calls, [("china.18".into(), "hours 3".into())]);
        assert_eq!(routes[0][1].calls, [("china.19".into(), "hours 3".into())]);
    }

    #[test]
    fn reads_autonomous_date_windows_without_promoting_triggered_only_events() {
        let trigger = "trigger = { tag = PRC date > 2024.1.1
                         has_completed_focus = PRC_test date < 2025.1.1 }";
        let script = format!("country_event = {{ fire_only_once = yes {trigger} }}");
        let timing = autonomous_date(&script).unwrap();
        assert_eq!(timing.tag.as_deref(), Some("PRC"));
        assert_eq!(timing.after.as_deref(), Some("2024.1.1"));
        assert_eq!(timing.before.as_deref(), Some("2025.1.1"));
        assert_eq!(timing.focuses, ["PRC_test"]);
        assert!(autonomous_date(&script.replace("fire_only_once", "is_triggered_only")).is_none());
        assert!(autonomous_date(&script.replace("has_completed_focus", "has_war_with")).is_none());
    }

    #[test]
    fn reads_autonomous_flag_conditions_and_a_simple_mean_delay() {
        let script = r#"country_event = {
            fire_only_once = yes
            trigger = { tag = SOV OR = { has_global_flag = one has_completed_focus = two } }
            mean_time_to_happen = { days = 230 }
        }"#;
        let trigger = autonomous_trigger(script, "SOV").unwrap();
        assert_eq!(trigger.owner, "SOV");
        assert_eq!(trigger.mean_days, Some(230));
        assert!(autonomous_trigger(
            &script.replace("days = 230", "days = 230 modifier = {}"),
            "SOV"
        )
        .is_none());
        assert!(autonomous_trigger(
            &script.replace("has_completed_focus", "controls_state"),
            "SOV"
        )
        .is_none());
    }

    #[test]
    fn parses_progress_threshold_and_keeps_bypass_separate_from_rewards() {
        let event = "country_event = { fire_only_once = yes trigger = { focus_progress = { focus = FRA_example progress > 0.5 } } }";
        let progress = focus_progress_trigger(event).unwrap();
        assert_eq!(progress.focus, "FRA_example");
        assert_eq!(progress.progress, 0.5);
        assert!(!progress.inclusive);
        assert!(focus_progress_trigger(&event.replace("progress >", "progress <")).is_none());
        let completed = block_focus_completions("completion_reward = { unlock_national_focus = FRA_skipped GER = { complete_national_focus = { focus = GER_completed } } }");
        assert_eq!(completed.len(), 2);
        assert!(completed[0].bypass);
        assert_eq!(completed[1].tag.as_deref(), Some("GER"));
        assert_eq!(completed[1].id, "GER_completed");
        assert!(!completed[1].bypass);
    }

    #[test]
    fn keeps_decision_activation_and_expiry_events_distinct() {
        let block = r#"decision = {
            days_remove = 14
            complete_effect = {
                if = { limit = { NOT = { has_country_flag = announced } } country_event = sample.1 }
                effect_tooltip = { country_event = sample.3 }
            }
            remove_effect = {
                if = { limit = { has_country_flag = announced } country_event = { id = sample.2 hours = 6 } }
            }
        }"#;
        let calls = decision_guarded_calls(block);
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].target, "sample.1");
        assert!(!calls[0].deferred);
        assert!(!calls[0].checks[0].present);
        assert_eq!(calls[1].target, "sample.2");
        assert_eq!(calls[1].delay, "days 14, hours 6");
        assert!(calls[1].deferred);
        assert!(calls[1].checks[0].present);
    }

    #[test]
    fn expands_scripted_effects_without_moving_them_outside_their_scopes() {
        let effects = ScriptedEffects(HashMap::from([(
            "send_followup".into(),
            "send_followup = { country_event = sample.2 }".into(),
        )]));
        let script = r#"immediate = {
            JAP = { send_followup = yes }
            if = { limit = { always = no } send_followup = yes }
            effect_tooltip = { send_followup = yes }
        }"#;
        let expanded = effects.expand_in_scope(script, 8, &mut 32);
        assert!(direct_event_calls(&expanded, false).is_empty());
        let calls = scoped_event_calls(&expanded, "PRC");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].recipient, "JAP");
        assert_eq!(calls[0].target, "sample.2");
    }

    #[test]
    fn omits_only_verified_shared_development_events() {
        let political = Path::new("events/TFR_events_ZZZ_political.txt");
        assert!(is_shared_development_notice(political, "generic.7"));
        assert!(is_shared_development_notice(political, "generic.16"));
        assert!(is_shared_development_notice(political, "generic.18"));
        assert!(!is_shared_development_notice(political, "generic.20"));
        assert!(!is_shared_development_notice(political, "jap.16"));
        assert!(!is_shared_development_notice(
            Path::new("events/other.txt"),
            "generic.7"
        ));
    }

    #[test]
    fn separates_country_recipients_and_global_news_calls() {
        let effect = r#"completion_reward = {
            hidden_effect = {
                JAP = { country_event = jap.8 }
                AST = { country_event = { id = jap.8 days = 1 } }
                news_event = { id = news.139 hours = 8 }
                if = { limit = { always = no } GER = { country_event = ger.99 } }
            }
            effect_tooltip = { FRA = { country_event = fra.99 } }
        }"#;
        assert_eq!(
            scoped_event_calls(effect, "PRC"),
            vec![
                ForeignEvent {
                    recipient: "AST".into(),
                    target: "jap.8".into(),
                    delay: "days 1".into()
                },
                ForeignEvent {
                    recipient: "JAP".into(),
                    target: "jap.8".into(),
                    delay: String::new()
                },
            ],
        );
        assert_eq!(
            direct_event_calls(effect, true),
            vec![("news.139".into(), "hours 8".into())]
        );
    }

    #[test]
    fn keeps_conditional_ending_text_and_localized_credits() {
        let loc = BTreeMap::from([
            ("ending.title".into(), "Epilogue".into()),
            (
                "ending.credits".into(),
                "§YThank you for playing!§! £GFX_ending".into(),
            ),
        ]);
        let text = r#"title = {
            text = ending.title
            trigger = { has_completed_focus = final_focus }
        }"#;
        let entries = direct_statements(text).collect::<Vec<_>>();
        let (fallback, variants) = event_text(&entries, "title", &loc);
        assert!(fallback.is_empty());
        assert_eq!(variants[0].completed_focuses, ["final_focus"]);
        assert_eq!(variants[0].text, "Epilogue");
        assert_eq!(
            ending_credits("option = { custom_effect_tooltip = ending.credits }", &loc),
            ["Thank you for playing!"],
        );
    }

    #[test]
    fn keeps_timed_mission_outcomes_separate_from_preview_tooltips() {
        let script = r#"mission = {
            complete_effect = { country_event = example.1 }
            timeout_effect = {
                hidden_effect = { JAP = { country_event = { id = example.2 days = 3 } } }
                effect_tooltip = { country_event = example.3 }
            }
            cancel_effect = { apply_outcome = yes }
        }"#;
        let effects = ScriptedEffects(HashMap::from([(
            "apply_outcome".into(),
            "apply_outcome = { country_event = example.4 }".into(),
        )]));
        let audio = super_events::Catalog::default();
        let completed = mission_effect(script, "complete_effect", "PRC", &effects, &audio);
        let timed_out = mission_effect(script, "timeout_effect", "PRC", &effects, &audio);
        let cancelled = mission_effect(script, "cancel_effect", "PRC", &effects, &audio);
        assert_eq!(completed.calls, vec![("example.1".into(), String::new())]);
        assert!(timed_out.calls.is_empty());
        assert_eq!(timed_out.foreign_events[0].recipient, "JAP");
        assert_eq!(timed_out.foreign_events[0].delay, "days 3");
        assert_eq!(cancelled.calls, vec![("example.4".into(), String::new())]);
    }
}
