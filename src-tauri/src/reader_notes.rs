use super::mod_layers;
use super::{block_body, blocks, decode, direct_statements, top_value, ScriptedEffects};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::{Path, PathBuf},
    sync::OnceLock,
};

pub fn inline_assets(root: &Path) -> BTreeMap<String, PathBuf> {
    let mut assets = BTreeMap::new();
    for entry in mod_layers::entries(root, "interface") {
        let file = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if !file.ends_with(".gfx") || !(file.contains("texticon") || file.contains("flag")) {
            continue;
        }
        let Ok(script) = decode(entry.path()) else {
            continue;
        };
        for sprite in blocks(&script, "spriteType = {") {
            if top_value(&sprite, "noOfFrames")
                .parse::<u32>()
                .is_ok_and(|n| n > 1)
            {
                continue;
            }
            let name = top_value(&sprite, "name");
            let texture = top_value(&sprite, "texturefile");
            if let Some(path) = mod_layers::file(root, &texture).filter(|_| !name.is_empty()) {
                assets.insert(name.to_ascii_lowercase(), path);
            }
        }
    }
    assets
}

pub fn text_colors(root: &Path, game: Option<&Path>) -> BTreeMap<String, String> {
    let mut colors = BTreeMap::new();
    let paths = game
        .into_iter()
        .map(|root| root.join("interface/core.gfx"))
        .chain(
            mod_layers::roots(root)
                .into_iter()
                .map(|root| root.join("interface/core.gfx")),
        );
    for path in paths {
        let Ok(script) = decode(&path) else {
            continue;
        };
        let Some(palette) = blocks(&script, "textcolors = {").into_iter().next() else {
            continue;
        };
        for (code, body) in direct_statements(block_body(&palette).unwrap_or_default()) {
            if code.len() != 1 {
                continue;
            }
            let values = block_body(body)
                .unwrap_or_default()
                .split_whitespace()
                .filter_map(|n| n.parse::<u8>().ok())
                .collect::<Vec<_>>();
            if let [r, g, b] = values.as_slice() {
                colors.insert(code.to_owned(), format!("#{r:02x}{g:02x}{b:02x}"));
            }
        }
    }
    colors
}

#[tauri::command]
pub async fn load_inline_flag(
    mod_path: String,
    game_path: Option<String>,
    tag: String,
) -> Option<String> {
    tauri::async_runtime::spawn_blocking(move || {
        if tag.len() != 3
            || !tag
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
        {
            return None;
        }
        super::country_flag(Path::new(&mod_path), &tag).or_else(|| {
            game_path
                .filter(|p| !p.is_empty())
                .and_then(|root| super::country_flag(Path::new(&root), &tag))
        })
    })
    .await
    .ok()
    .flatten()
}

#[derive(Deserialize, Serialize)]
pub struct OpeningNote {
    text: String,
    images: Vec<PathBuf>,
}

fn text_icons() -> &'static Regex {
    static ICON: OnceLock<Regex> = OnceLock::new();
    ICON.get_or_init(|| Regex::new(r"£([A-Za-z0-9_]+)(?:\|\d+)?").unwrap())
}

fn plain(text: &str) -> String {
    static COLOR: OnceLock<Regex> = OnceLock::new();
    let color = COLOR.get_or_init(|| Regex::new(r"§[A-Za-z0-9!?]").unwrap());
    text_icons()
        .replace_all(&color.replace_all(text, ""), "")
        .trim()
        .to_owned()
}

pub fn opening_notes(
    root: &Path,
    loc: &BTreeMap<String, String>,
) -> BTreeMap<String, Vec<OpeningNote>> {
    let keys = [
        "start_menu_country_desc",
        "start_menu_country_paths",
        "start_menu_country_other",
    ];
    let mut entries = Vec::new();
    for path in mod_layers::entries(root, "common/scripted_localisation")
        .map(|entry| entry.into_path())
        .filter(|path| {
            path.extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("txt"))
        })
    {
        let Ok(script) = decode(&path) else { continue };
        if !keys.iter().any(|key| script.contains(key)) {
            continue;
        }
        for definition in blocks(&script, "defined_text = {") {
            let Some(order) = keys
                .iter()
                .position(|key| *key == top_value(&definition, "name"))
            else {
                continue;
            };
            for entry in blocks(&definition, "text = {") {
                let trigger = blocks(&entry, "trigger = {");
                let [trigger] = trigger.as_slice() else {
                    continue;
                };
                let checks =
                    direct_statements(block_body(trigger).unwrap_or_default()).collect::<Vec<_>>();
                let [("tag", tag)] = checks.as_slice() else {
                    continue;
                };
                if let Some(text) = loc.get(&top_value(&entry, "localization_key")) {
                    entries.push((tag.trim_matches('"').to_owned(), order, text));
                }
            }
        }
    }
    let icons = entries
        .iter()
        .flat_map(|(_, _, text)| {
            text_icons()
                .captures_iter(text)
                .map(|cap| cap.get(1).unwrap().as_str())
        })
        .collect::<HashSet<_>>();
    let mut images = HashMap::new();
    if !icons.is_empty() {
        for path in mod_layers::entries(root, "interface")
            .map(|entry| entry.into_path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("gfx"))
            })
        {
            let Ok(script) = decode(&path) else { continue };
            if !icons.iter().any(|name| script.contains(name)) {
                continue;
            }
            for sprite in blocks(&script, "spriteType = {") {
                let name = top_value(&sprite, "name");
                if icons.contains(name.as_str()) {
                    if let Some(texture) =
                        mod_layers::file(root, &top_value(&sprite, "texturefile"))
                    {
                        images.insert(name, texture);
                    }
                }
            }
        }
    }
    entries.sort_by_key(|(_, order, _)| *order);
    let mut notes = BTreeMap::<_, Vec<_>>::new();
    for (tag, _, source) in entries {
        let text = plain(source);
        let images = text_icons()
            .captures_iter(source)
            .filter_map(|cap| images.get(&cap[1]).cloned())
            .collect::<Vec<_>>();
        if !text.is_empty() || !images.is_empty() {
            notes
                .entry(tag)
                .or_default()
                .push(OpeningNote { text, images });
        }
    }
    notes
}

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct EffectPreview {
    lines: Vec<EffectLine>,
    incomplete: bool,
}

#[derive(Clone, Deserialize, Serialize)]
struct EffectLine {
    text: String,
    context: Vec<String>,
}

impl EffectPreview {
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty() && !self.incomplete
    }
}

pub fn game_template_paths<'a>(
    game: &Path,
    language: &'a str,
) -> impl Iterator<Item = PathBuf> + 'a {
    let directory = game.join("localisation");
    ["english", "simp_chinese", "traditional_chinese", "japanese"]
        .into_iter()
        .filter(move |lang| *lang == "english" || *lang == language)
        .map(move |lang| directory.join(lang).join(format!("effects_l_{lang}.yml")))
}

pub fn game_templates(game: Option<&Path>, language: &str) -> BTreeMap<String, String> {
    game.into_iter()
        .flat_map(|path| game_template_paths(path, language))
        .filter_map(|path| decode(&path).ok())
        .flat_map(|source| super::loc_parse(&source))
        .collect()
}

pub fn option_effects(
    block: &str,
    loc: &BTreeMap<String, String>,
    templates: &BTreeMap<String, String>,
    effects: &ScriptedEffects,
) -> EffectPreview {
    let mut reader = EffectReader {
        loc,
        templates,
        effects,
        preview: EffectPreview::default(),
        budget: 160,
    };
    reader.walk(block, &[], 8);
    reader.preview
}

struct EffectReader<'a> {
    loc: &'a BTreeMap<String, String>,
    templates: &'a BTreeMap<String, String>,
    effects: &'a ScriptedEffects,
    preview: EffectPreview,
    budget: usize,
}

impl EffectReader<'_> {
    fn name(&self, key: &str) -> String {
        plain(
            self.loc
                .get(key)
                .or_else(|| self.loc.get(&format!("{key}_DEF")))
                .map_or(key, String::as_str),
        )
    }

    fn line(&mut self, text: String, context: &[String]) {
        if !text.is_empty() {
            self.preview.lines.push(EffectLine {
                text,
                context: context.to_vec(),
            });
        }
    }

    fn description(&self, key: &str) -> Option<String> {
        static REFERENCE: OnceLock<Regex> = OnceLock::new();
        let reference =
            REFERENCE.get_or_init(|| Regex::new(r"\$([A-Za-z0-9_.-]+)(?:\|[^$]*)?\$").unwrap());
        let mut text = self.loc.get(key)?.clone();
        for _ in 0..4 {
            let next = reference.replace_all(&text, |cap: &regex::Captures<'_>| {
                self.loc
                    .get(&cap[1])
                    .cloned()
                    .unwrap_or_else(|| cap[0].to_owned())
            });
            if next == text {
                break;
            }
            text = next.into_owned();
        }
        Some(text.trim().to_owned())
    }

    fn template(
        &mut self,
        key: &str,
        fallback: &str,
        values: &[(&str, String)],
        context: &[String],
    ) {
        static PARAM: OnceLock<Regex> = OnceLock::new();
        let param = PARAM.get_or_init(|| Regex::new(r"\$([A-Z_]+)(?:\|([^$]*))?\$").unwrap());
        let template = self
            .loc
            .get(key)
            .or_else(|| self.templates.get(key))
            .map_or(fallback, String::as_str);
        let mut missing = false;
        let text = param.replace_all(template, |cap: &regex::Captures<'_>| {
            if let Some((_, value)) = values.iter().find(|(name, _)| *name == &cap[1]) {
                let style = cap.get(2).map_or("", |value| value.as_str());
                if let Ok(mut number) = value.parse::<f64>() {
                    if style.contains('%') {
                        number *= 100.0;
                    }
                    let digits = style
                        .chars()
                        .find_map(|c| c.to_digit(10))
                        .map_or(2, |d| d.min(3)) as usize;
                    let mut formatted = format!("{number:.digits$}");
                    if formatted.contains('.') {
                        formatted = formatted
                            .trim_end_matches('0')
                            .trim_end_matches('.')
                            .to_owned();
                    }
                    if style.contains('+') && number > 0.0 {
                        formatted.insert(0, '+');
                    }
                    if style.contains('%') {
                        formatted.push('%');
                    }
                    if style.contains('+') && number != 0.0 {
                        let positive = (number > 0.0) != style.contains('-');
                        let color = if positive { 'G' } else { 'R' };
                        return format!("§{color}{formatted}§!");
                    }
                    if let Some(color) = style.chars().find(|c| c.is_ascii_alphabetic()) {
                        return format!("§{color}{formatted}§!");
                    }
                    return formatted;
                }
                if let Some(color) = style.chars().find(|c| c.is_ascii_alphabetic()) {
                    return format!("§{color}{value}§!");
                }
                return value.clone();
            }
            missing = true;
            String::new()
        });
        if missing {
            self.preview.incomplete = true;
        } else {
            self.line(text.trim().to_owned(), context);
        }
    }

    fn walk(&mut self, block: &str, context: &[String], depth: usize) {
        if depth == 0 || self.budget == 0 {
            self.preview.incomplete = true;
            return;
        }
        for (command, argument) in direct_statements(block_body(block).unwrap_or_default()) {
            if self.budget == 0 {
                self.preview.incomplete = true;
                break;
            }
            self.budget -= 1;
            let value = argument.trim_matches('"');
            let field = |name| top_value(argument, name);
            match command {
                "name"
                | "trigger"
                | "limit"
                | "ai_chance"
                | "highlight"
                | "default"
                | "modifier"
                | "hidden_effect"
                | "hidden_trigger"
                | "sound_effect"
                | "scoped_sound_effect"
                | "play_song"
                | "scoped_play_song"
                | "set_country_flag"
                | "clr_country_flag"
                | "set_global_flag"
                | "clr_global_flag"
                | "set_temp_variable"
                | "set_variable"
                | "add_to_variable"
                | "subtract_from_variable"
                | "log" => continue,
                "custom_effect_tooltip" | "custom_tooltip" => {
                    let key = if argument.starts_with('{') {
                        field("localization_key")
                    } else {
                        value.to_owned()
                    };
                    if let Some(text) = self.description(&key) {
                        self.preview.incomplete |= text.contains('[') || text.contains('$');
                        self.line(text, context);
                    } else {
                        self.preview.incomplete = true;
                    }
                }
                "effect_tooltip" | "show_effect_tooltip" => self.walk(argument, context, depth - 1),
                "if" | "else_if" | "else" | "random" | "random_list" | "random_country"
                | "random_owned_state" | "every_country" | "every_owned_state" => {
                    let marker = match command {
                        "else" => "@otherwise",
                        "random" | "random_list" | "random_country" | "random_owned_state" => {
                            "@random"
                        }
                        "every_country" | "every_owned_state" => "@scoped",
                        _ => "@conditional",
                    };
                    let mut nested = context.to_vec();
                    let limit = blocks(argument, "limit = {");
                    let checks = limit
                        .first()
                        .and_then(|block| block_body(block))
                        .map(|body| direct_statements(body).collect::<Vec<_>>())
                        .unwrap_or_default();
                    let mission = match checks.as_slice() {
                        [("has_active_mission", mission)] => mission.trim_matches('"'),
                        _ => "",
                    };
                    nested.push(if !mission.is_empty() && marker == "@conditional" {
                        format!("@active-mission:{}", self.name(mission))
                    } else {
                        marker.into()
                    });
                    self.walk(argument, &nested, depth - 1);
                }
                _ => {
                    if let Some((key, fallback)) = numeric_effect(command) {
                        if value.parse::<f64>().is_ok_and(f64::is_finite) {
                            self.template(
                                key,
                                fallback,
                                &[("VAL", value.into()), ("COUNT", value.into())],
                                context,
                            );
                        } else {
                            self.preview.incomplete = true;
                        }
                    } else if let Some((key, fallback)) = named_effect(command) {
                        let key_name = if matches!(
                            command,
                            "country_event" | "news_event" | "unit_leader_event"
                        ) && argument.starts_with('{')
                        {
                            field("id")
                        } else {
                            value.to_owned()
                        };
                        if command.ends_with("_event") {
                            let title = [format!("{key_name}.t"), format!("{key_name}.title")]
                                .iter()
                                .find_map(|key| self.loc.get(key))
                                .map_or_else(|| self.name(&key_name), |title| plain(title));
                            self.template(key, fallback, &[("EVENT", title)], context);
                        } else if argument.starts_with('{') {
                            let body = block_body(argument).unwrap_or_default();
                            let ids = body
                                .lines()
                                .flat_map(|line| super::clean_script_line(line).split_whitespace())
                                .collect::<Vec<_>>();
                            if !ids.iter().all(|id| {
                                id.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
                            }) {
                                self.preview.incomplete = true;
                                continue;
                            }
                            for id in ids {
                                self.template(
                                    key,
                                    fallback,
                                    &[("NAME", self.name(id)), ("EFFECT", String::new())],
                                    context,
                                );
                            }
                        } else {
                            self.template(
                                key,
                                fallback,
                                &[("NAME", self.name(&key_name)), ("EFFECT", String::new())],
                                context,
                            );
                        }
                    } else if command == "add_equipment_to_stockpile" {
                        let (kind, amount) = (field("type"), field("amount"));
                        if !kind.is_empty() && amount.parse::<i64>().is_ok() {
                            self.template(
                                "EFFECT_ADD_EQUIPMENT_TO_STOCKPILE",
                                "$EQUIPMENT$: $AMOUNT|+0$",
                                &[("EQUIPMENT", self.name(&kind)), ("AMOUNT", amount)],
                                context,
                            );
                        } else {
                            self.preview.incomplete = true;
                        }
                    } else if command == "add_days_mission_timeout" {
                        let (mission, days) = (field("mission"), field("days"));
                        if !mission.is_empty() && days.parse::<i32>().is_ok() {
                            self.template(
                                "HOI4READER_MISSION_DAYS",
                                "$NAME$: $DAYS|+0$ days",
                                &[("NAME", self.name(&mission)), ("DAYS", days)],
                                context,
                            );
                        } else {
                            self.preview.incomplete = true;
                        }
                    } else if command == "add_building_construction"
                        || command == "add_offsite_building"
                    {
                        let (kind, level) = (field("type"), field("level"));
                        if !kind.is_empty() && level.parse::<i32>().is_ok() {
                            let key = if command == "add_offsite_building" {
                                "EFFECT_ADD_OFFSITE_BUILDING"
                            } else if field("instant_build") == "yes" {
                                "EFFECT_ADD_BUILDING_CONSTRUCTION_INSTANT"
                            } else {
                                "EFFECT_ADD_BUILDING_CONSTRUCTION"
                            };
                            self.template(
                                key,
                                "$BUILDING$: $NUMBER|+0$",
                                &[("BUILDING", self.name(&kind)), ("NUMBER", level)],
                                context,
                            );
                        } else {
                            self.preview.incomplete = true;
                        }
                    } else if let Some(script) = self
                        .effects
                        .0
                        .get(command)
                        .filter(|_| value == "yes" || argument.starts_with('{'))
                    {
                        let mut expanded = script.clone();
                        if argument.starts_with('{') {
                            for (key, val) in
                                direct_statements(block_body(argument).unwrap_or_default())
                            {
                                expanded =
                                    expanded.replace(&format!("${key}$"), val.trim_matches('"'));
                            }
                        }
                        self.walk(&expanded, context, depth - 1);
                    } else if argument.starts_with('{')
                        && (command.bytes().all(|c| c.is_ascii_digit()) || is_scope(command))
                    {
                        let mut nested = context.to_vec();
                        nested.push(if command.bytes().all(|c| c.is_ascii_digit()) {
                            "@scoped".into()
                        } else {
                            self.name(command)
                        });
                        self.walk(argument, &nested, depth - 1);
                    } else {
                        self.preview.incomplete = true;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[test]
fn option_preview_keeps_tooltips_but_not_hidden_effects() {
    let loc = BTreeMap::from([("bonus_note".into(), "§YReserve bonus§!".into())]);
    let templates = BTreeMap::from([(
        "EFFECT_MANPOWER_ADD_COUNTRY".into(),
        "Manpower: $VAL|+=0$.".into(),
    )]);
    let effects = ScriptedEffects(HashMap::from([(
        "scripted_bonus".into(),
        "{ add_war_support = $bonus$ }".into(),
    )]));
    let preview = option_effects(
        r#"option = {
        name = example.a
        effect_tooltip = { add_manpower = 1 }
        custom_effect_tooltip = bonus_note
        hidden_effect = { add_manpower = 500 }
        if = { limit = { has_active_mission = timer } add_days_mission_timeout = { mission = timer days = -5 } }
        scripted_bonus = { bonus = 0.025 }
        add_stability = var:unknown
    }"#,
        &loc,
        &templates,
        &effects,
    );
    let lines = preview
        .lines
        .iter()
        .map(|line| plain(&line.text))
        .collect::<Vec<_>>();
    assert_eq!(
        lines,
        [
            "Manpower: +1.",
            "Reserve bonus",
            "timer: -5 days",
            "War support: +2.5%"
        ]
    );
    assert_eq!(preview.lines[2].context, ["@active-mission:timer"]);
    assert!(preview.incomplete);
}

fn is_scope(command: &str) -> bool {
    matches!(command, "ROOT" | "FROM" | "PREV" | "THIS")
        || (command.len() == 3
            && command
                .bytes()
                .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()))
        || command.starts_with("event_target:")
}

fn numeric_effect(command: &str) -> Option<(&'static str, &'static str)> {
    Some(match command {
        "add_manpower" => ("EFFECT_MANPOWER_ADD_COUNTRY", "Manpower: $VAL|+0$"),
        "add_political_power" => ("POLITICS_ADD_POLITICAL_POWER", "Political power: $VAL|+0$"),
        "set_political_power" => (
            "POLITICS_SET_POLITICAL_POWER",
            "Set political power to $VAL$",
        ),
        "add_command_power" => ("EFFECT_ADD_COMMAND_POWER", "Command power: $VAL|+1$"),
        "add_stability" => ("EFFECT_ADD_STABILITY", "Stability: $VAL|+%$"),
        "add_war_support" => ("EFFECT_ADD_WAR_SUPPORT", "War support: $VAL|+%$"),
        "add_army_experience" => ("EFFECT_ADD_ARMY_EXPERIENCE", "Army experience: $VAL|+0$"),
        "add_navy_experience" => ("EFFECT_ADD_NAVY_EXPERIENCE", "Navy experience: $VAL|+0$"),
        "add_air_experience" => ("EFFECT_ADD_AIR_EXPERIENCE", "Air experience: $VAL|+0$"),
        "add_fuel" => ("POLITICS_ADD_FUEL", "Fuel: $VAL|+0$"),
        "set_fuel" => ("EFFECT_SET_FUEL", "Set fuel to $VAL$"),
        "add_research_slot" => ("EFFECT_ADD_RESEARCH_SLOT", "Research slots: $COUNT|+0$"),
        "set_research_slots" => ("EFFECT_SET_RESEARCH_SLOTS", "Set research slots to $COUNT$"),
        "add_convoys" => ("EFFECT_ADD_CONVOYS", "Convoys: $VAL|+0$"),
        _ => return None,
    })
}

fn named_effect(command: &str) -> Option<(&'static str, &'static str)> {
    Some(match command {
        "add_ideas" | "add_idea" => ("EFFECT_ADD_IDEA", "Gain $NAME$"),
        "remove_ideas" | "remove_idea" => ("EFFECT_REMOVE_IDEA", "Lose $NAME$"),
        "activate_mission" => ("EFFECT_ACTIVATES_MISSION", "Activate mission: $NAME$"),
        "remove_mission" => ("EFFECT_REMOVE_MISSION", "Remove mission: $NAME$"),
        "activate_decision" => ("EFFECT_ACTIVATE_DECISION", "Activate decision: $NAME$"),
        "unlock_decision" => ("EFFECT_UNLOCK_DECISION", "Unlock decision: $NAME$"),
        "remove_decision" => ("EFFECT_REMOVE_DECISION", "Remove decision: $NAME$"),
        "country_event" | "news_event" | "unit_leader_event" => {
            ("EFFECT_COUNTRY_EVENT", "Event: $EVENT$")
        }
        _ => return None,
    })
}
