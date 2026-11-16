use super::{
    block_body, blocks, decode, direct_statements, localisation_for, named_blocks,
    script_condition_in_country, script_values, top_value, ScriptCondition, STANDARD,
};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    path::{Path, PathBuf},
};

const TNO_NUCLEAR_SHEET: (u32, u32) = (573 * 28, 455);

#[derive(Clone, Deserialize, Serialize)]
pub struct Scene {
    id: String,
    title: String,
    quote: Option<String>,
    dismiss: Option<String>,
    image_path: Option<String>,
    audio: Option<Track>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Track {
    title: String,
    path: String,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum MusicEffect {
    Play {
        track: Track,
        recipient: Option<String>,
    },
    Branch {
        arms: Vec<MusicArm>,
    },
}

#[derive(Deserialize, Serialize, Clone)]
pub struct MusicArm {
    condition: ScriptCondition,
    owner: Option<String>,
    effects: Vec<MusicEffect>,
}

#[derive(Default)]
struct FlagScene {
    title: String,
    quote: String,
    dismiss: String,
    image: Option<PathBuf>,
}

#[derive(Default)]
pub struct Catalog {
    flags: HashMap<String, FlagScene>,
    sprites: HashMap<String, PathBuf>,
    music: HashMap<String, PathBuf>,
    sounds: HashMap<String, PathBuf>,
    sound_aliases: HashMap<String, String>,
    titles: HashMap<String, String>,
    decision_music_after_gui: HashMap<String, Vec<MusicEffect>>,
    dynamic_tno_audio: BTreeSet<String>,
    tno: bool,
}

pub(super) fn local_file(base: &Path, value: &str) -> Option<PathBuf> {
    if value.is_empty() {
        return None;
    }
    let root = base.canonicalize().ok()?;
    let relative = value.replace('\\', "/");
    let path = base.join(&relative);
    #[cfg(not(windows))]
    let path = if path.is_file() {
        path
    } else {
        case_insensitive_asset_path(base, Path::new(&relative))?
    };
    let resolved = path.canonicalize().ok()?;
    (resolved.starts_with(root) && resolved.is_file()).then_some(path)
}

#[cfg(not(windows))]
fn case_insensitive_asset_path(base: &Path, relative: &Path) -> Option<PathBuf> {
    let mut path = base.to_path_buf();
    for component in relative.components() {
        let std::path::Component::Normal(name) = component else {
            return None;
        };
        let exact = path.join(name);
        if exact.exists() {
            path = exact;
            continue;
        }
        let name = name.to_string_lossy();
        let mut matching = std::fs::read_dir(&path)
            .ok()?
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().eq_ignore_ascii_case(&name));
        let matched = matching.next()?;
        if matching.next().is_some() {
            return None;
        }
        path = matched.path();
    }
    Some(path)
}

fn inner(block: &str) -> &str {
    block
        .find('{')
        .zip(block.rfind('}'))
        .and_then(|(start, end)| block.get(start + 1..end))
        .unwrap_or_default()
}

impl Catalog {
    pub fn load(
        root: &Path,
        language: &str,
        loc: &BTreeMap<String, String>,
        diagnostics: &mut Vec<String>,
    ) -> Self {
        let mut catalog = Self::default();
        let tfr_loc = super::mod_layers::path(root, "common/scripted_localisation/TFR_Super_Event_Scripted_Loc.txt");
        let tno_effect = super::mod_layers::path(root, "common/scripted_effects/TNO_super_events_scripted_effect.txt");
        let tfr = tfr_loc.is_file();
        if let Ok(script) = decode(&tno_effect) {
            catalog.tno = true;
            for key in ["if = {", "else_if = {"] {
                for clause in blocks(&script, key) {
                    // Look at each conditional clause, including those nested under the
                    // game-rule guard. Only the actual token comparisons carry a token.
                    for condition in blocks(&clause, "check_variable = {") {
                        let token = top_value(&condition, "TNO_temp_super_event");
                        if let Some(token) = token.strip_prefix("token:") {
                            if script_values(&clause, "scoped_play_song")
                                .iter()
                                .any(|song| song.starts_with(&format!("TNO_{token}_")))
                            {
                                catalog.dynamic_tno_audio.insert(token.to_string());
                            }
                        }
                    }
                }
            }
        }
        for name in ["TFR_Super_Events.gfx", "TNO_SG_Super_Event.gfx", "TNOLAR_SG_Super_Event.gfx"] {
            let Ok(script) = decode(&super::mod_layers::path(root, &format!("interface/{name}"))) else {
                continue;
            };
            for sprite in blocks(&script, "spriteType = {")
                .into_iter()
                .chain(blocks(&script, "FrameAnimatedSpriteType = {"))
            {
                let name = top_value(&sprite, "name");
                let mut texture = top_value(&sprite, "texturefile");
                if texture.is_empty() {
                    texture = top_value(&sprite, "textureFile");
                }
                if let Some(path) = super::mod_layers::file(root, &texture) {
                    catalog.sprites.insert(name, path);
                }
            }
        }

        if let Ok(script) = decode(&tfr_loc) {
            for text in blocks(&script, "defined_text = {") {
                let kind = top_value(&text, "name");
                for entry in blocks(&text, "text = {") {
                    let flags = Self::flags(&entry, "has");
                    if flags.len() != 1 {
                        continue;
                    }
                    let scene = catalog.flags.entry(flags[0].clone()).or_default();
                    let key = top_value(&entry, "localization_key");
                    match kind.as_str() {
                        "get_super_event_title" => scene.title = key,
                        "get_superevent_quote" => scene.quote = key,
                        "get_superevent_buton" => scene.dismiss = key,
                        _ => {}
                    }
                }
            }
            catalog.flags.retain(|_, scene| !scene.title.is_empty());
        }

        if tfr {
            let gui = decode(&super::mod_layers::path(root, "interface/TFR_Super_Events.gui")).unwrap_or_default();
            let windows = blocks(&gui, "containerWindowType = {")
                .into_iter()
                .filter_map(|window| {
                    let sprite = blocks(&window, "background = {")
                        .first()
                        .map(|background| top_value(background, "spriteType"))?;
                    Some((
                        top_value(&window, "name"),
                        catalog.sprites.get(&sprite)?.clone(),
                    ))
                })
                .collect::<HashMap<_, _>>();
            let gui =
                decode(&super::mod_layers::path(root, "common/scripted_guis/TFR_super_events.txt")).unwrap_or_default();
            for group in blocks(&gui, "scripted_gui = {") {
                for (_, entry) in named_blocks(inner(&group)) {
                    let window = top_value(&entry, "window_name");
                    let Some(image) = windows.get(&window) else {
                        continue;
                    };
                    for condition in blocks(&entry, "visible = {") {
                        for flag in Self::flags(&condition, "has") {
                            if let Some(scene) = catalog.flags.get_mut(&flag) {
                                scene.image = Some(image.clone());
                            }
                        }
                    }
                }
            }
        }

        // Steam installs the official soundtrack as a separate Workshop item.
        if tfr {
            let companion = root
                .parent()
                .map(|parent| parent.join("3350892196"))
                .filter(|path| path.join("music").is_dir());
            if let Some(companion) = companion {
                catalog.load_audio_assets(&companion, "music");
                let companion_loc = localisation_for(&companion, language, diagnostics);
                catalog.load_titles(&companion_loc);
            } else {
                diagnostics.push("The Fire Rises Music Addon (3350892196) is missing from this Workshop library. Some scripted tracks are unavailable.".into());
            }
        }
        catalog.load_audio_assets(root, "music");
        catalog.load_audio_assets(root, "sound");
        if root.file_name().is_some_and(|id| id == super::mod_layers::TFR_CHILD) {
            let companion = root.with_file_name("3793754397");
            if companion.is_dir() {
                catalog.load_audio_assets(&companion, "music");
                catalog.load_titles(&localisation_for(&companion, language, diagnostics));
            }
        }
        catalog.load_titles(loc);
        if tfr {
            // The reader has no in-game time machine window. Its closing cue
            // belongs to the decision that opens that window.
            let gui = decode(&super::mod_layers::path(root, "common/scripted_guis/TFR_scripted_guis_JAP.txt"))
                .unwrap_or_default();
            for (_, window) in blocks(&gui, "scripted_gui = {")
                .iter()
                .flat_map(|script| named_blocks(inner(script)))
                .filter(|(key, _)| key == "TFR_JAP_time_travel")
            {
                let visible = blocks(&window, "visible = {");
                if visible
                    .first()
                    .is_none_or(|block| top_value(block, "has_country_flag") != "time_travel_flag")
                {
                    continue;
                }
                for close in blocks(&window, "close_button_click = {") {
                    if top_value(&close, "clr_country_flag") == "time_travel_flag" {
                        catalog
                            .decision_music_after_gui
                            .insert("time_travel_flag".into(), catalog.music_effects(&close));
                    }
                }
            }
        }
        catalog
    }

    pub fn decision_music(&self, scope: &str) -> Vec<MusicEffect> {
        let mut effects = self.music_effects(scope);
        for (name, flag) in super::effect_commands(scope) {
            if name == "set_country_flag" {
                effects.extend(
                    self.decision_music_after_gui
                        .get(flag)
                        .into_iter()
                        .flatten()
                        .cloned(),
                );
            }
        }
        effects
    }

    fn load_titles(&mut self, loc: &BTreeMap<String, String>) {
        for (key, text) in loc {
            let id = key.to_ascii_lowercase();
            if self.music.contains_key(&id)
                || self.sounds.contains_key(&id)
                || self.sound_aliases.contains_key(&id)
            {
                if let Some(title) = track_title(text) {
                    self.titles.insert(id, title);
                }
            }
        }
    }

    fn load_audio_assets(&mut self, root: &Path, directory: &str) {
        for entry in super::mod_layers::entries(root, directory)
            .filter(|entry| {
                entry.file_type().is_file()
                    && entry
                        .path()
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("asset"))
            })
        {
            let Ok(script) = decode(entry.path()) else {
                continue;
            };
            let base = entry.path().parent().unwrap_or(root);
            let (marker, index) = if directory == "music" {
                ("music = {", &mut self.music)
            } else {
                ("sound = {", &mut self.sounds)
            };
            for asset in blocks(&script, marker) {
                let name = top_value(&asset, "name");
                let file = top_value(&asset, "file");
                if let Some(path) = local_file(base, &file) {
                    index.insert(name.to_ascii_lowercase(), path);
                }
            }
            if directory == "sound" {
                for effect in blocks(&script, "soundeffect = {") {
                    let sounds = script_values(&effect, "sound");
                    if sounds.len() == 1 {
                        self.sound_aliases.insert(
                            top_value(&effect, "name").to_ascii_lowercase(),
                            sounds[0].to_ascii_lowercase(),
                        );
                    }
                }
            }
        }
    }

    pub fn music_effects(&self, scope: &str) -> Vec<MusicEffect> {
        if !scope.contains("play_song") {
            return Vec::new();
        }
        self.music_in_scope(scope, None, 12)
    }

    fn music_in_scope(&self, scope: &str, owner: Option<&str>, depth: u8) -> Vec<MusicEffect> {
        if depth == 0 {
            return Vec::new();
        }
        let mut results = Vec::new();
        let mut chain = Vec::<MusicArm>::new();
        let flush = |chain: &mut Vec<MusicArm>, results: &mut Vec<MusicEffect>| {
            if chain.iter().any(|arm| !arm.effects.is_empty()) {
                results.push(MusicEffect::Branch {
                    arms: std::mem::take(chain),
                });
            } else {
                chain.clear();
            }
        };
        for (key, value) in direct_statements(block_body(scope).unwrap_or_default()) {
            let command = key.to_ascii_lowercase();
            if !matches!(command.as_str(), "else_if" | "else") {
                flush(&mut chain, &mut results);
            }
            match command.as_str() {
                "scoped_play_song" | "play_song" => {
                    let id = value.trim_matches('"').to_ascii_lowercase();
                    if !id.contains("silence") {
                        if let Some(track) = self.track(&id, self.music.get(&id), None) {
                            results.push(MusicEffect::Play {
                                track,
                                recipient: owner.map(str::to_string),
                            });
                        }
                    }
                }
                "hidden_effect" => results.extend(self.music_in_scope(value, owner, depth - 1)),
                "if" | "else_if" | "else" if command == "if" || !chain.is_empty() => {
                    let condition = if command == "else" {
                        ScriptCondition::All(Vec::new())
                    } else {
                        direct_statements(block_body(value).unwrap_or_default())
                            .find_map(|(key, value)| {
                                key.eq_ignore_ascii_case("limit")
                                    .then(|| block_body(value))
                                    .flatten()
                            })
                            .map(|limit| {
                                script_condition_in_country(limit, 8, owner.unwrap_or_default())
                            })
                            .unwrap_or(ScriptCondition::Unknown)
                    };
                    chain.push(MusicArm {
                        condition,
                        owner: owner.map(str::to_string),
                        effects: self.music_in_scope(value, owner, depth - 1),
                    });
                }
                _ if key.len() == 3
                    && key
                        .bytes()
                        .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
                    && !matches!(key, "NOT" | "AND" | "NOR") =>
                {
                    results.extend(self.music_in_scope(value, Some(key), depth - 1));
                }
                _ => {}
            }
        }
        flush(&mut chain, &mut results);
        results
    }

    fn flags(scope: &str, command: &str) -> Vec<String> {
        ["country", "global"]
            .into_iter()
            .flat_map(|scope_type| {
                let prefix = format!("{scope_type}:");
                script_values(scope, &format!("{command}_{scope_type}_flag"))
                    .into_iter()
                    .map(move |flag| format!("{prefix}{flag}"))
            })
            .collect()
    }

    pub fn resolve<'a>(
        &self,
        scopes: impl IntoIterator<Item = &'a str>,
        loc: &BTreeMap<String, String>,
    ) -> Option<Scene> {
        let scopes = scopes.into_iter().collect::<Vec<_>>();
        let mut matches = BTreeMap::new();
        for scope in scopes {
            if self.tno
                && scope.contains("TNO_fire_super_event")
                && script_values(scope, "TNO_fire_super_event")
                    .iter()
                    .any(|value| value == "yes")
            {
                for variable in blocks(scope, "set_temp_variable = {") {
                    let token = top_value(&variable, "TNO_temp_super_event");
                    let Some(id) = token.strip_prefix("token:SE_") else {
                        continue;
                    };
                    let key = format!("SE_{id}");
                    let track = format!("TNO_{key}");
                    matches.insert(
                        key.clone(),
                        Scene {
                            id: key.clone(),
                            title: super::localized(loc, &key),
                            quote: loc.get(&format!("{key}_D")).cloned(),
                            dismiss: loc.get(&format!("{key}_A")).cloned(),
                            image_path: self
                                .sprites
                                .get(&format!("GFX_{key}"))
                                .map(|p| p.display().to_string()),
                            audio: (!self.dynamic_tno_audio.contains(&key))
                                .then(|| {
                                    self.track(
                                        &track,
                                        self.music.get(&track.to_ascii_lowercase()),
                                        None,
                                    )
                                })
                                .flatten(),
                        },
                    );
                }
            }
            if self.flags.is_empty() || !scope.contains("set_") {
                continue;
            }
            for id in Self::flags(scope, "set") {
                let Some(scene) = self.flags.get(&id) else {
                    continue;
                };
                matches.insert(
                    id.clone(),
                    Scene {
                        id,
                        title: super::localized(loc, &scene.title),
                        quote: loc.get(&scene.quote).cloned(),
                        dismiss: loc.get(&scene.dismiss).cloned(),
                        image_path: scene.image.as_ref().map(|p| p.display().to_string()),
                        audio: self.tfr_track(scope, loc.get(&scene.title).map(String::as_str)),
                    },
                );
            }
        }
        (matches.len() == 1)
            .then(|| matches.into_values().next())
            .flatten()
    }

    fn tfr_track(&self, scope: &str, scene_title: Option<&str>) -> Option<Track> {
        // TFR often plays the actual cue as a sound effect alongside a silent song.
        for (commands, is_sound) in [
            (["sound_effect", "scoped_sound_effect"], true),
            (["play_song", "scoped_play_song"], false),
        ] {
            let tracks = commands
                .into_iter()
                .flat_map(|key| script_values(scope, key))
                .filter_map(|name| {
                    let id = name.to_ascii_lowercase();
                    if id.contains("silence") {
                        return None;
                    }
                    let path = if is_sound {
                        self.sounds.get(self.sound_aliases.get(&id).unwrap_or(&id))
                    } else {
                        self.music.get(&id)
                    };
                    self.track(&id, path, if is_sound { scene_title } else { None })
                        .map(|track| (track.path.clone(), track))
                })
                .collect::<BTreeMap<_, _>>();
            match tracks.len() {
                0 => {}
                1 => return tracks.into_values().next(),
                _ => return None,
            }
        }
        None
    }

    fn track(&self, key: &str, path: Option<&PathBuf>, scene_title: Option<&str>) -> Option<Track> {
        let path = path?;
        let title = self
            .titles
            .get(&key.to_ascii_lowercase())
            .cloned()
            .or_else(|| scene_title.and_then(track_title))
            .unwrap_or_else(|| key.to_string());
        Some(Track {
            title,
            path: path.display().to_string(),
        })
    }
}

fn track_title(text: &str) -> Option<String> {
    static MARKUP: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let markup = MARKUP
        .get_or_init(|| regex::Regex::new(r"§[A-Za-z0-9!?]|£[A-Za-z0-9_]+(?:\|\d+)?").unwrap());
    let clean = markup.replace_all(text, "");
    clean
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(str::to_string)
}

fn decode_dds(bytes: &mut [u8]) -> Option<image::DynamicImage> {
    if bytes.get(..4)? != b"DDS " {
        return None;
    }
    let height = u32::from_le_bytes(bytes.get(12..16)?.try_into().ok()?);
    let width = u32::from_le_bytes(bytes.get(16..20)?.try_into().ok()?);
    if (width, height) != TNO_NUCLEAR_SHEET
        && (!(1..=8192).contains(&width) || !(1..=8192).contains(&height))
    {
        return None;
    }
    let dds_u32 = |offset| Some(u32::from_le_bytes(bytes.get(offset..offset + 4)?.try_into().ok()?));
    let bgra_offset: Option<usize> = match bytes.get(84..88)? {
        // TFR's main menu uses DXGI_FORMAT_B8G8R8A8_UNORM_SRGB.
        b"DX10" if matches!(dds_u32(128)?, 87 | 91) && dds_u32(132)? == 3 => Some(148),
        // The original super-event button declares BGRA via legacy channel masks.
        b"\0\0\0\0"
            if [
                dds_u32(88)?,
                dds_u32(92)?,
                dds_u32(96)?,
                dds_u32(100)?,
                dds_u32(104)?,
            ] == [32, 0x00ff_0000, 0x0000_ff00, 0x0000_00ff, 0xff00_0000] =>
        {
            Some(128)
        }
        _ => None,
    };
    if let Some(offset) = bgra_offset {
        let pitch = dds_u32(20)? as usize;
        let row_bytes = width as usize * 4;
        if pitch >= row_bytes {
            let data = bytes.get(offset..offset.checked_add(pitch.checked_mul(height as usize)?)?)?;
            let mut rgba = Vec::with_capacity(row_bytes * height as usize);
            for row in data.chunks_exact(pitch) {
                for pixel in row[..row_bytes].chunks_exact(4) {
                    rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
                }
            }
            return image::RgbaImage::from_raw(width, height, rgba)
                .map(image::DynamicImage::ImageRgba8);
        }
    }
    if matches!(bytes.get(84..88)?, b"DXT1" | b"DXT3" | b"DXT5") {
        // The file already contains complete compression blocks at the edges.
        // Decode those blocks, then crop back to the authored canvas.
        bytes[12..16].copy_from_slice(&height.next_multiple_of(4).to_le_bytes());
        bytes[16..20].copy_from_slice(&width.next_multiple_of(4).to_le_bytes());
    }
    image::load_from_memory_with_format(bytes, image::ImageFormat::Dds)
        .ok()
        .map(|decoded| decoded.crop_imm(0, 0, width, height))
}

#[cfg(test)]
#[test]
fn decodes_the_bgra_dds_format_used_by_the_tfr_menu() {
    let mut bytes = [0; 156];
    bytes[..4].copy_from_slice(b"DDS ");
    bytes[84..88].copy_from_slice(b"DX10");
    for (offset, value) in [(12, 1u32), (16, 1), (20, 8), (128, 91), (132, 3)] {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    bytes[148..152].copy_from_slice(&[11, 22, 33, 255]);
    let image = decode_dds(&mut bytes).unwrap().into_rgba8();
    assert_eq!(image.get_pixel(0, 0).0, [33, 22, 11, 255]);
}

#[cfg(test)]
#[test]
fn decodes_the_legacy_bgra_dds_format_used_by_the_tfr_super_event_button() {
    let mut bytes = [0; 132];
    bytes[..4].copy_from_slice(b"DDS ");
    for (offset, value) in [
        (12, 1u32),
        (16, 1),
        (20, 4),
        (88, 32),
        (92, 0x00ff_0000),
        (96, 0x0000_ff00),
        (100, 0x0000_00ff),
        (104, 0xff00_0000),
    ] {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    bytes[128..132].copy_from_slice(&[11, 22, 33, 128]);
    let image = decode_dds(&mut bytes).unwrap().into_rgba8();
    assert_eq!(image.get_pixel(0, 0).0, [33, 22, 11, 128]);
}

fn decode_scene_image(mod_path: &Path, image_path: &Path) -> Option<String> {
    let gfx = mod_path.join("gfx").canonicalize().ok()?;
    let path = image_path.canonicalize().ok()?;
    if !path.starts_with(gfx) || path.metadata().ok()?.len() > 32 * 1024 * 1024 {
        return None;
    }
    let mut bytes = std::fs::read(path).ok()?;
    let mut decoded = if bytes.starts_with(b"DDS ") {
        decode_dds(&mut bytes)?
    } else {
        image::load_from_memory(&bytes).ok()?
    };
    if image_path.file_name().is_some_and(|name| name.eq_ignore_ascii_case("nuclearwar.dds"))
        && (decoded.width(), decoded.height()) == TNO_NUCLEAR_SHEET
    {
        decoded = decoded.crop_imm(0, 0, 573, 455);
    }
    // Sprite slices are measured in source pixels, so resize only oversized artwork.
    let image = if decoded.width() > 1600 || decoded.height() > 1000 {
        decoded.thumbnail(1600, 1000)
    } else {
        decoded
    };
    png_data_url(&image)
}

fn png_data_url(image: &image::DynamicImage) -> Option<String> {
    let mut png = std::io::Cursor::new(Vec::new());
    image.write_to(&mut png, image::ImageFormat::Png).ok()?;
    Some(format!(
        "data:image/png;base64,{}",
        STANDARD.encode(png.into_inner())
    ))
}

#[derive(Serialize)]
pub struct TnoSuperEventSkin {
    frame: Option<String>,
    title: Option<String>,
    quote: Option<String>,
    button: Option<String>,
    nuclear_frame: Option<String>,
    nuclear_title: Option<String>,
    quote_font: Option<PathBuf>,
}

#[tauri::command]
pub async fn load_tno_super_event_skin(mod_path: String) -> Option<TnoSuperEventSkin> {
    tauri::async_runtime::spawn_blocking(move || {
        let root = Path::new(&mod_path);
        super::mod_layers::file(root, "interface/TNO_SG_Super_Event.gui")?;
        let layers = super::mod_layers::roots(root);
        let image = |relative: &str| {
            let path = super::mod_layers::file(root, relative)?;
            layers.iter().rev().find_map(|layer| decode_scene_image(layer, &path))
        };
        Some(TnoSuperEventSkin {
            frame: image("gfx/interface/tiles/tiled_window_bigevent_border.dds"),
            title: image("gfx/interface/tiles/tiled_bg.dds"),
            quote: image("gfx/interface/superevent_text_underlay.dds"),
            button: image("gfx/interface/button_221x34.dds"),
            nuclear_frame: image("gfx/interface/topbar/nuke_super_frame.dds"),
            nuclear_title: image("gfx/interface/topbar/nuke_super_title.dds"),
            quote_font: super::mod_layers::file(root, "gfx/fonts/BombardierTNOV6.otf"),
        })
    })
    .await
    .ok()
    .flatten()
}

#[derive(Serialize)]
struct TnoGlyph {
    image: String,
    advance: f32,
    width: f32,
    height: f32,
    left: f32,
    bottom: f32,
}

#[derive(Serialize, Default)]
pub struct TnoSuperEventGlyphs {
    title: BTreeMap<char, TnoGlyph>,
    quote: BTreeMap<char, TnoGlyph>,
}

fn bitmap_field(line: &str, field: &str) -> Option<i32> {
    line.split_ascii_whitespace()
        .filter_map(|word| word.split_once('='))
        .find_map(|(name, value)| (name == field).then(|| value.parse().ok()).flatten())
}

fn load_tno_glyph_page(
    root: &Path,
    stem: &str,
    page: i32,
    wanted: &BTreeSet<char>,
) -> Option<BTreeMap<char, TnoGlyph>> {
    if wanted.is_empty() {
        return None;
    }
    let relative = format!("gfx/fonts/{stem}");
    let metadata = std::fs::read_to_string(local_file(root, &format!("{relative}.fnt"))?).ok()?;
    let size = bitmap_field(metadata.lines().next()?, "size")?.unsigned_abs() as f32;
    let baseline = bitmap_field(
        metadata.lines().find(|line| line.starts_with("common "))?,
        "base",
    )?;
    if size == 0.0 {
        return None;
    }
    let rows: Vec<_> = metadata
        .lines()
        .filter_map(|line| {
            let id = line
                .strip_prefix("char id=")?
                .split_ascii_whitespace()
                .next()?;
            let glyph = char::from_u32(id.parse().ok()?)?;
            wanted.contains(&glyph).then_some((glyph, line))
        })
        .collect();
    if rows.is_empty() {
        return None;
    }
    // The publisher uses the font stem for the DDS even when the descriptor names it with _0.
    let path = local_file(root, &format!("{relative}.dds"))
        .or_else(|| local_file(root, &format!("{relative}_{page}.dds")))?;
    let mut bytes = std::fs::read(path).ok()?;
    let sheet = decode_dds(&mut bytes)?;
    Some(
        rows.into_iter()
            .filter_map(|(glyph, line)| {
                let metric = |name| bitmap_field(line, name);
                if metric("page")? != page {
                    return None;
                }
                let x = u32::try_from(metric("x")?).ok()?;
                let y = u32::try_from(metric("y")?).ok()?;
                let width = u32::try_from(metric("width")?).ok()?;
                let height = u32::try_from(metric("height")?).ok()?;
                if !(1..=64).contains(&width)
                    || !(1..=64).contains(&height)
                    || x.checked_add(width)? > sheet.width()
                    || y.checked_add(height)? > sheet.height()
                {
                    return None;
                }
                Some((
                    glyph,
                    TnoGlyph {
                        image: png_data_url(&sheet.crop_imm(x, y, width, height))?,
                        advance: metric("xadvance")? as f32 / size,
                        width: width as f32 / size,
                        height: height as f32 / size,
                        left: metric("xoffset")? as f32 / size,
                        bottom: (baseline as f32 - metric("yoffset")? as f32 - height as f32)
                            / size,
                    },
                ))
            })
            .collect(),
    )
}

#[tauri::command]
pub async fn load_tno_super_event_glyphs(
    mod_path: String,
    title: String,
    quote_and_button: String,
) -> TnoSuperEventGlyphs {
    tauri::async_runtime::spawn_blocking(move || {
        let wanted = |text: &str| -> BTreeSet<char> {
            text.chars()
                .filter(|ch| matches!(*ch, '\u{2e80}'..='\u{9fff}' | '\u{f900}'..='\u{faff}' | '\u{ff00}'..='\u{ffef}'))
                .take(800)
                .collect()
        };
        let roots = super::mod_layers::localisation_roots(Path::new(&mod_path));
        let load = |stem, page, wanted| roots.iter().rev()
            .find_map(|root| load_tno_glyph_page(root, stem, page, wanted))
            .unwrap_or_default();
        let title_text = wanted(&title);
        let mut title = load("fzrzhdb_20_outline_0", 0, &title_text);
        title.extend(load("fzrzhdb_20_outline_1", 1, &title_text));
        TnoSuperEventGlyphs {
            title,
            quote: load("fzwhjz_22", 0, &wanted(&quote_and_button)),
        }
    }).await.unwrap_or_default()
}

#[tauri::command]
pub async fn load_tfr_menu_art(mod_path: String) -> Option<String> {
    tauri::async_runtime::spawn_blocking(move || {
        let root = Path::new(&mod_path);
        let interface = root.join("interface");
        if !interface.join("TFR_start_menu.gui").is_file()
            && !interface.join("NVX_menu.gfx").is_file()
        {
            return None;
        }
        for file in ["frontendmainviewbg.gfx", "NVX_menu.gfx"] {
            let Ok(script) = decode(&interface.join(file)) else {
                continue;
            };
            for sprite in blocks(&script, "corneredTileSpriteType = {") {
                if top_value(&sprite, "name") != "GFX_frontend_bg" {
                    continue;
                }
                let texture = top_value(&sprite, "texturefile");
                if let Some(image) = local_file(root, &texture)
                    .and_then(|path| decode_scene_image(root, &path))
                {
                    return Some(image);
                }
            }
        }
        decode_scene_image(root, &root.join("gfx/loadingscreens/main_menu.dds"))
    })
    .await
    .ok()
    .flatten()
}

#[derive(Serialize)]
pub struct TfrSuperEventSkin {
    frame: Option<String>,
    button: Option<String>,
}

#[tauri::command]
pub async fn load_tfr_super_event_skin(mod_path: String) -> Option<TfrSuperEventSkin> {
    tauri::async_runtime::spawn_blocking(move || {
        let root = Path::new(&mod_path);
        let script = decode(&root.join("interface/TFR_Super_Events.gfx")).ok()?;
        let mut skin = TfrSuperEventSkin {
            frame: None,
            button: None,
        };
        for sprite in blocks(&script, "spriteType = {") {
            let target = match top_value(&sprite, "name").as_str() {
                "GFX_Super_event_bg" => &mut skin.frame,
                "GFX_spacebar" => &mut skin.button,
                _ => continue,
            };
            let texture = top_value(&sprite, "texturefile");
            *target = local_file(root, &texture).and_then(|path| decode_scene_image(root, &path));
            if skin.frame.is_some() && skin.button.is_some() {
                break;
            }
        }
        Some(skin)
    })
    .await
    .ok()
    .flatten()
}

pub(super) fn with_event_art(mut scene: Scene, event_art: Option<&PathBuf>) -> Scene {
    if scene.image_path.is_none() {
        scene.image_path = event_art.map(|path| path.display().to_string());
    }
    scene
}

#[tauri::command]
pub async fn load_super_event_image(mod_path: String, image_path: String) -> Option<String> {
    tauri::async_runtime::spawn_blocking(move || {
        super::mod_layers::roots(Path::new(&mod_path)).iter().rev()
            .find_map(|root| decode_scene_image(root, Path::new(&image_path)))
    })
    .await
    .ok()
    .flatten()
}

#[cfg(test)]
#[test]
fn conditional_music_uses_the_asset_title_and_preserves_the_else_arm() {
    let mut catalog = Catalog::default();
    catalog
        .music
        .insert("dregs".into(), "audio/dregs.ogg".into());
    catalog
        .music
        .insert("korogaru".into(), "audio/korogaru.ogg".into());
    catalog.load_titles(&BTreeMap::from([
        ("dregs".into(), "§Yたんたんたん§!\n\n£GFX_album_art".into()),
        ("korogaru".into(), "転がる岩、君に朝が降る".into()),
    ]));
    let effects = catalog.music_effects(r#"immediate = {
        if = { limit = { has_global_flag = JAP_japan_lost } hidden_effect = { scoped_play_song = dregs } }
        ELSE = { scoped_play_song = korogaru }
        effect_tooltip = { scoped_play_song = korogaru }
    }"#);
    let [MusicEffect::Branch { arms }] = effects.as_slice() else {
        panic!("expected one branch")
    };
    assert_eq!(arms.len(), 2);
    assert!(
        matches!(&arms[0].effects[0], MusicEffect::Play { track, .. } if track.title == "たんたんたん")
    );
    assert!(
        matches!(&arms[1].effects[0], MusicEffect::Play { track, .. } if track.title == "転がる岩、君に朝が降る")
    );
}

#[cfg(test)]
#[test]
fn decodes_partial_dxt5_edge_blocks_at_original_size() {
    let mut bytes = vec![0; 128 + 16];
    bytes[..4].copy_from_slice(b"DDS ");
    for (offset, value) in [
        (4, 124u32),
        (8, 0x81007),
        (12, 2),
        (16, 3),
        (20, 16),
        (76, 32),
        (80, 4),
        (108, 0x1000),
    ] {
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
    }
    bytes[84..88].copy_from_slice(b"DXT5");
    let image = decode_dds(&mut bytes).expect("padded DXT5 blocks should decode");
    assert_eq!((image.width(), image.height()), (3, 2));
}
