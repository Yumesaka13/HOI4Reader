use eframe::egui;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Lang {
    En,
    ZhHans,
    ZhHant,
    Ja,
}
impl Default for Lang {
    fn default() -> Self {
        Self::ZhHans
    }
}
impl Lang {
    fn code(self) -> &'static str {
        match self {
            Self::En => "english",
            Self::ZhHans => "simp_chinese",
            Self::ZhHant => "traditional_chinese",
            Self::Ja => "japanese",
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::ZhHans => "简体中文",
            Self::ZhHant => "繁體中文",
            Self::Ja => "日本語",
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Page {
    Home,
    Mods,
    Countries,
    Chapters,
    Reader,
    Settings,
}
#[derive(Clone, Serialize, Deserialize, Default)]
struct Saved {
    game: String,
    mods_root: String,
    mod_path: String,
    language: Lang,
    tag: String,
    focus_id: String,
}
#[derive(Clone)]
struct Focus {
    id: String,
    title: String,
    desc: String,
    icon: String,
    image: Option<PathBuf>,
    source: String,
}
struct Reader {
    saved: Saved,
    page: Page,
    mods: Vec<(String, String)>,
    countries: Vec<(String, String)>,
    focuses: Vec<Focus>,
    locs: BTreeMap<String, String>,
    focus_index: usize,
    status: String,
    diagnostics: Vec<String>,
    texture: Option<egui::TextureHandle>,
}
const KNOWN: &[(&str, &str)] = &[
    ("2438003901", "The New Order: Last Days of Europe (TNO)"),
    ("3350890356", "The Fire Rises (TFR)"),
    ("1521695605", "Kaiserreich"),
    ("2777392649", "Millennium Dawn"),
    ("820260968", "Road to 56"),
    ("2265420196", "Old World Blues"),
    ("1821967568", "Equestria at War"),
    ("2819909272", "Kaiserredux"),
];
fn appdata() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."))
        })
        .join("HOI4Reader")
}
fn save(s: &Saved) {
    let d = appdata();
    let _ = fs::create_dir_all(&d);
    let _ = fs::write(
        d.join("reading.json"),
        serde_json::to_vec_pretty(s).unwrap_or_default(),
    );
}
fn load() -> Saved {
    fs::read(appdata().join("reading.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}
fn parse_loc(text: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for l in text.trim_start_matches('\u{feff}').lines() {
        let l = l.trim();
        if l.is_empty() || l.starts_with('#') || l.starts_with("l_") {
            continue;
        }
        if let Some((k, v)) = l.split_once(':') {
            let val = v
                .trim()
                .trim_start_matches(|c: char| c.is_ascii_digit() || c == ' ' || c == '\"')
                .trim_matches('\"')
                .replace("\\n", "\n")
                .replace("\\\"", "\"");
            out.insert(k.trim().to_owned(), val);
        }
    }
    out
}
fn known_name(id: &str, desc: &Path) -> String {
    for (i, n) in KNOWN {
        if id == *i {
            return n.to_string();
        }
    }
    fs::read_to_string(desc)
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
fn scan_mods(root: &Path) -> Vec<(String, String)> {
    let mut out = vec![];
    if !root.exists() {
        return out;
    }
    for e in WalkDir::new(root)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_dir())
    {
        let id = e.file_name().to_string_lossy().to_string();
        let descriptor = e.path().join("descriptor.mod");
        if descriptor.exists() || KNOWN.iter().any(|(x, _)| *x == id) {
            out.push((known_name(&id, &descriptor), e.path().display().to_string()))
        }
    }
    out.sort_by_key(|x| x.0.to_lowercase());
    out
}
fn steam_roots() -> Vec<PathBuf> {
    let mut roots = vec![];
    #[cfg(windows)]
    {
        for d in b'C'..=b'Z' {
            for p in [
                format!("{}:\\Program Files (x86)\\Steam", d as char),
                format!("{}:\\Program Files\\Steam", d as char),
                format!("{}:\\Programs\\Steam", d as char),
                format!("{}:\\SteamLibrary", d as char),
            ] {
                let p = PathBuf::from(p);
                if p.join("steamapps").exists() {
                    roots.push(p)
                }
            }
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
                    if let Some(x) = e.path().parent() {
                        roots.push(x.to_path_buf())
                    }
                }
            }
        }
    }
    #[cfg(target_os = "macos")]
    {
        if let Some(h) = std::env::var_os("HOME") {
            roots.push(PathBuf::from(h).join("Library/Application Support/Steam"));
        }
    }
    roots
}
fn discover() -> (String, Vec<(String, String)>, String) {
    let roots = steam_roots();
    let (mut game, mut mods, mut workshop) = (String::new(), vec![], String::new());
    for root in &roots {
        let g = root.join("steamapps/common/Hearts of Iron IV");
        if g.join("common").exists() {
            game = g.display().to_string()
        }
        let w = root.join("steamapps/workshop/content/394360");
        if w.exists() {
            workshop = w.display().to_string();
            mods.extend(scan_mods(&w));
        }
        if let Ok(v) = fs::read_to_string(root.join("steamapps/libraryfolders.vdf")) {
            for line in v.lines() {
                let t = line.trim();
                if t.starts_with("\"path\"") {
                    if let Some(p) = t.split('"').nth(3) {
                        let lib = PathBuf::from(p.replace("\\\\", "\\"));
                        let gp = lib.join("steamapps/common/Hearts of Iron IV");
                        if game.is_empty() && gp.join("common").exists() {
                            game = gp.display().to_string()
                        }
                        let wp = lib.join("steamapps/workshop/content/394360");
                        if wp.exists() {
                            workshop = wp.display().to_string();
                            mods.extend(scan_mods(&wp));
                        }
                    }
                }
            }
        }
    }
    mods.sort_by_key(|x| x.1.clone());
    mods.dedup_by(|a, b| a.1 == b.1);
    (game, mods, workshop)
}
fn read_tree(root: &Path, lang: Lang) -> (BTreeMap<String, String>, Vec<String>) {
    let mut map = BTreeMap::new();
    let mut diag = vec![];
    let loc = root.join("localisation");
    if !loc.exists() {
        return (
            map,
            vec![format!("Missing localisation directory: {}", loc.display())],
        );
    }
    for e in WalkDir::new(loc)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_type().is_file()
                && e.path()
                    .extension()
                    .is_some_and(|x| x.eq_ignore_ascii_case("yml"))
                && e.file_name()
                    .to_string_lossy()
                    .to_lowercase()
                    .contains(lang.code())
        })
    {
        match fs::read(e.path()) {
            Ok(b) => {
                let s = String::from_utf8_lossy(&b);
                map.extend(parse_loc(&s))
            }
            Err(x) => diag.push(format!("{}: {x}", e.path().display())),
        }
    }
    (map, diag)
}
fn val_after(block: &str, key: &str) -> String {
    for l in block.lines() {
        let t = l.trim();
        if let Some(v) = t
            .strip_prefix(key)
            .and_then(|x| x.trim_start().strip_prefix('='))
        {
            return v.trim().trim_matches('"').to_string();
        }
    }
    String::new()
}
fn extract_blocks(text: &str, marker: &str) -> Vec<String> {
    let mut out = vec![];
    let mut from = 0;
    while let Some(pos) = text[from..].find(marker) {
        let start = from + pos;
        let open = match text[start..].find('{') {
            Some(x) => start + x,
            None => break,
        };
        let mut depth = 0i32;
        let (mut quoted, mut escape) = (false, false);
        let mut end = None;
        for (i, ch) in text[open..].char_indices() {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch == '"' {
                quoted = !quoted;
                continue;
            }
            if !quoted {
                if ch == '{' {
                    depth += 1
                }
                if ch == '}' {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(open + i + 1);
                        break;
                    }
                }
            }
        }
        if let Some(e) = end {
            out.push(text[start..e].to_string());
            from = e
        } else {
            break;
        }
    }
    out
}
fn focus_data(root: &Path, loc: &BTreeMap<String, String>, tag: &str) -> (Vec<Focus>, Vec<String>) {
    let mut items = vec![];
    let mut diag = vec![];
    let dir = root.join("common/national_focus");
    if !dir.exists() {
        diag.push(format!("Missing national_focus: {}", dir.display()));
        return (items, diag);
    }
    for file in WalkDir::new(&dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_type().is_file()
                && e.path()
                    .extension()
                    .is_some_and(|x| x.eq_ignore_ascii_case("txt"))
        })
    {
        let text = match fs::read_to_string(file.path()) {
            Ok(s) => s,
            Err(e) => {
                diag.push(format!("{}: {e}", file.path().display()));
                continue;
            }
        };
        for block in extract_blocks(&text, "focus = {") {
            let id = val_after(&block, "id");
            if id.is_empty() {
                continue;
            }
            let prefix = id.split('_').next().unwrap_or("");
            if !tag.is_empty() && prefix != tag && !id.starts_with(&format!("{tag}_")) {
                continue;
            }
            let title = loc
                .get(&id)
                .cloned()
                .unwrap_or_else(|| format!("[Missing translation: {id}]"));
            let desc = loc.get(&format!("{id}_desc")).cloned().unwrap_or_default();
            let icon = val_after(&block, "icon");
            let image = if icon.is_empty() {
                None
            } else {
                find_icon(root, &icon)
            };
            items.push(Focus {
                id,
                title,
                desc,
                icon,
                image,
                source: file.path().display().to_string(),
            });
        }
    }
    items.sort_by(|a, b| a.title.cmp(&b.title));
    if items.is_empty() {
        diag.push(format!("No national focus entries matched country {tag}. Tree association is heuristic based on focus id prefix; inspect source diagnostics."));
    }
    (items, diag)
}
fn find_icon(root: &Path, icon: &str) -> Option<PathBuf> {
    let key = icon.trim_start_matches("GFX_");
    let roots = [
        root.join("gfx/interface/goals"),
        root.join("gfx/interface").join("goals"),
    ];
    for r in roots {
        if !r.exists() {
            continue;
        }
        for e in WalkDir::new(r)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                e.file_type().is_file()
                    && e.path()
                        .extension()
                        .is_some_and(|x| x.eq_ignore_ascii_case("png"))
            })
        {
            if e.path()
                .file_stem()
                .is_some_and(|x| x.to_string_lossy().eq_ignore_ascii_case(key))
            {
                return Some(e.path().to_path_buf());
            }
        }
    }
    None
}
fn load_texture(ctx: &egui::Context, p: &Path) -> Option<egui::TextureHandle> {
    let bytes = fs::read(p).ok()?;
    let img = image::load_from_memory(&bytes).ok()?.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    Some(ctx.load_texture(
        p.display().to_string(),
        egui::ColorImage::from_rgba_unmultiplied(size, &img),
        egui::TextureOptions::LINEAR,
    ))
}
fn cjk_fonts(c: &egui::Context) {
    let mut f = egui::FontDefinitions::default();
    for p in [
        "C:/Windows/Fonts/NotoSansSC-VF.ttf",
        "C:/Windows/Fonts/NotoSansJP-VF.ttf",
        "C:/Windows/Fonts/meiryo.ttc",
        "/System/Library/Fonts/PingFang.ttc",
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
    ] {
        if let Ok(b) = fs::read(p) {
            let n = format!("cjk{}", f.font_data.len());
            f.font_data.insert(n.clone(), egui::FontData::from_owned(b));
            f.families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, n.clone());
            f.families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, n);
        }
    }
    c.set_fonts(f)
}
fn primary_button(ui: &mut egui::Ui, label: &str) -> egui::Response {
    ui.add(
        egui::Button::new(egui::RichText::new(label).color(egui::Color32::WHITE))
            .fill(egui::Color32::from_rgb(103, 80, 164))
            .stroke(egui::Stroke::NONE)
            .rounding(egui::Rounding::same(20.0))
            .min_size(egui::vec2(120.0, 48.0)),
    )
}
fn material(c: &egui::Context) {
    // Material 3 light roles: purple seed, with explicit on-colors and surface levels.
    let primary = egui::Color32::from_rgb(103, 80, 164);
    let on_primary = egui::Color32::WHITE;
    let primary_container = egui::Color32::from_rgb(234, 221, 255);
    let on_primary_container = egui::Color32::from_rgb(33, 0, 93);
    let surface = egui::Color32::from_rgb(254, 247, 255);
    let surface_low = egui::Color32::from_rgb(247, 242, 250);
    let surface_container = egui::Color32::from_rgb(243, 237, 247);
    let surface_high = egui::Color32::from_rgb(236, 230, 240);
    let on_surface = egui::Color32::from_rgb(29, 27, 32);
    let outline = egui::Color32::from_rgb(121, 116, 126);
    let mut v = egui::Visuals::light();
    v.panel_fill = surface;
    v.window_fill = surface_container;
    v.extreme_bg_color = egui::Color32::WHITE;
    v.faint_bg_color = surface_low;
    v.code_bg_color = surface_high;
    v.override_text_color = Some(on_surface);
    v.selection.bg_fill = primary_container;
    v.selection.stroke = egui::Stroke::new(1.0_f32, on_primary_container);
    v.window_stroke = egui::Stroke::new(1.0_f32, outline);
    v.window_rounding = egui::Rounding::same(16.0);
    v.menu_rounding = egui::Rounding::same(12.0);
    for w in [&mut v.widgets.noninteractive, &mut v.widgets.inactive] {
        w.bg_fill = surface_low;
        w.weak_bg_fill = surface_low;
        w.fg_stroke = egui::Stroke::new(1.0_f32, on_surface);
        w.bg_stroke = egui::Stroke::NONE;
        w.rounding = egui::Rounding::same(12.0);
    }
    v.widgets.hovered.bg_fill = surface_high;
    v.widgets.hovered.weak_bg_fill = surface_high;
    v.widgets.hovered.fg_stroke = egui::Stroke::new(1.0_f32, on_surface);
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0_f32, outline);
    v.widgets.hovered.rounding = egui::Rounding::same(12.0);
    v.widgets.active.bg_fill = primary;
    v.widgets.active.weak_bg_fill = primary;
    v.widgets.active.fg_stroke = egui::Stroke::new(1.0_f32, on_primary);
    v.widgets.active.bg_stroke = egui::Stroke::new(1.0_f32, primary);
    v.widgets.active.rounding = egui::Rounding::same(12.0);
    v.widgets.open.bg_fill = primary_container;
    v.widgets.open.weak_bg_fill = primary_container;
    v.widgets.open.fg_stroke = egui::Stroke::new(1.0_f32, on_primary_container);
    v.widgets.open.rounding = egui::Rounding::same(12.0);
    v.widgets.noninteractive.fg_stroke.color = on_surface;
    let mut s = egui::Style::default();
    s.visuals = v;
    s.spacing.item_spacing = egui::vec2(12.0, 12.0);
    s.spacing.button_padding = egui::vec2(16.0, 10.0);
    s.spacing.interact_size = egui::vec2(48.0, 48.0);
    s.spacing.window_margin = egui::Margin::same(24.0);
    s.spacing.menu_margin = egui::Margin::same(8.0);
    s.text_styles
        .insert(egui::TextStyle::Body, egui::FontId::proportional(16.0));
    s.text_styles
        .insert(egui::TextStyle::Button, egui::FontId::proportional(14.0));
    s.text_styles
        .insert(egui::TextStyle::Heading, egui::FontId::proportional(24.0));
    s.text_styles
        .insert(egui::TextStyle::Small, egui::FontId::proportional(12.0));
    s.text_styles
        .insert(egui::TextStyle::Monospace, egui::FontId::monospace(14.0));
    c.set_style(s);
}
impl Reader {
    fn init() -> Self {
        let mut saved = load();
        let (g, m, w) = discover();
        if saved.game.is_empty() {
            saved.game = g
        }
        if saved.mods_root.is_empty() || !Path::new(&saved.mods_root).exists() {
            saved.mods_root = w
        }
        let mut r = Self {
            saved,
            page: Page::Home,
            mods: m,
            countries: vec![],
            focuses: vec![],
            locs: BTreeMap::new(),
            focus_index: 0,
            status: String::new(),
            diagnostics: vec![],
            texture: None,
        };
        r.reload();
        r
    }
    fn reload(&mut self) {
        if self.saved.mod_path.is_empty() {
            return;
        }
        let root = PathBuf::from(&self.saved.mod_path);
        let (m, d) = read_tree(&root, self.saved.language);
        self.locs = m;
        self.diagnostics = d;
        let mut tags = BTreeSet::new();
        for k in self.locs.keys() {
            if let Some(t) = k.strip_suffix("_ADJ").or_else(|| k.strip_suffix("_DEF")) {
                if t.len() >= 2 && t.len() <= 4 && !t.contains('_') {
                    tags.insert(t.to_string());
                }
            }
        }
        self.countries = tags
            .into_iter()
            .map(|t| {
                let name = self
                    .locs
                    .get(&format!("{t}_DEF"))
                    .or_else(|| self.locs.get(&format!("{t}_ADJ")))
                    .cloned()
                    .unwrap_or_else(|| t.clone());
                (t, name)
            })
            .collect();
        let (f, d) = focus_data(&root, &self.locs, &self.saved.tag);
        self.focuses = f;
        self.diagnostics.extend(d);
        self.focus_index = self
            .focuses
            .iter()
            .position(|x| x.id == self.saved.focus_id)
            .unwrap_or(0);
    }
    fn choose_mod(&mut self, path: String) {
        self.saved.mod_path = path;
        self.saved.tag.clear();
        self.saved.focus_id.clear();
        self.reload();
        self.page = Page::Countries;
        save(&self.saved)
    }
    fn choose_country(&mut self, tag: String) {
        self.saved.tag = tag;
        self.saved.focus_id.clear();
        self.reload();
        self.page = Page::Chapters;
        save(&self.saved)
    }
    fn select_focus(&mut self, i: usize, ctx: &egui::Context) {
        self.focus_index = i;
        if let Some(f) = self.focuses.get(i) {
            self.saved.focus_id = f.id.clone();
            self.texture = f.image.as_ref().and_then(|p| load_texture(ctx, p));
        }
        self.page = Page::Reader;
        save(&self.saved)
    }
}
impl eframe::App for Reader {
    fn update(&mut self, c: &egui::Context, _: &mut eframe::Frame) {
        egui::SidePanel::left("nav")
            .resizable(false)
            .exact_width(240.0)
            .frame(
                egui::Frame::default()
                    .fill(egui::Color32::from_rgb(247, 242, 250))
                    .inner_margin(egui::Margin::same(16.0)),
            )
            .show(c, |ui| {
                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new("HOI4Reader")
                        .size(22.0)
                        .color(egui::Color32::from_rgb(33, 0, 93)),
                );
                ui.add_space(24.0);
                for (p, label) in [(Page::Home, "⌂    主页"), (Page::Settings, "⚙    设置")]
                {
                    let active = self.page == p;
                    let fill = if active {
                        egui::Color32::from_rgb(234, 221, 255)
                    } else {
                        egui::Color32::TRANSPARENT
                    };
                    let color = if active {
                        egui::Color32::from_rgb(33, 0, 93)
                    } else {
                        egui::Color32::from_rgb(29, 27, 32)
                    };
                    let item = egui::Button::new(egui::RichText::new(label).color(color))
                        .fill(fill)
                        .stroke(egui::Stroke::NONE)
                        .rounding(egui::Rounding::same(28.0));
                    if ui.add_sized([208.0, 56.0], item).clicked() {
                        self.page = p;
                    }
                }
                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    ui.label(
                        egui::RichText::new("仅读取本机游戏文件")
                            .size(12.0)
                            .color(egui::Color32::from_rgb(73, 69, 79)),
                    );
                });
            });
        egui::CentralPanel::default().show(c, |ui| {
            ui.add_space(18.);
            match self.page {
                Page::Home => {
                    ui.heading("继续阅读");
                    ui.add_space(12.);
                    if !self.saved.focus_id.is_empty() {
                        if let Some(f) = self.focuses.iter().find(|f| f.id == self.saved.focus_id) {
                            egui::Frame::group(ui.style())
                                .fill(egui::Color32::from_rgb(247, 242, 250))
                                .stroke(egui::Stroke::NONE)
                                .rounding(egui::Rounding::same(16.0))
                                .inner_margin(24.)
                                .show(ui, |ui| {
                                    ui.label(&self.saved.mod_path);
                                    ui.heading(&f.title);
                                    ui.label(format!(
                                        "{} · {}",
                                        self.saved.tag,
                                        self.saved.language.label()
                                    ));
                                    if primary_button(ui, "继续阅读").clicked() {
                                        self.page = Page::Reader;
                                    }
                                });
                        }
                    } else {
                        egui::Frame::group(ui.style()).fill(egui::Color32::from_rgb(247, 242, 250)).stroke(egui::Stroke::NONE).rounding(egui::Rounding::same(28.0)).inner_margin(32.0).show(ui, |ui| { ui.set_max_width(760.0); ui.label(egui::RichText::new("开始一段新的阅读").size(28.0).color(egui::Color32::from_rgb(33, 0, 93))); ui.add_space(8.0); ui.label("选择已安装的模组，再按国家与国策章节逐步阅读剧情。阅读位置会自动保存。"); ui.add_space(24.0); if primary_button(ui, "浏览已安装模组").clicked() { self.page = Page::Mods; } });
                    }
                    ui.add_space(20.);
                    if !self.saved.mod_path.is_empty() && ui.button("继续选择国家").clicked()
                    {
                        self.page = Page::Countries;
                    }
                }
                Page::Mods => {
                    ui.heading("选择模组");
                    ui.label(format!("{} 个已安装模组", self.mods.len()));
                    egui::ScrollArea::vertical().id_salt("mods").show(ui, |ui| {
                        for (n, p) in self.mods.clone() {
                            if ui.selectable_label(self.saved.mod_path == p, &n).clicked() {
                                self.choose_mod(p)
                            }
                        }
                    });
                    if self.mods.is_empty() {
                        ui.label("未发现创意工坊模组。请在设置中检查 Steam Workshop 目录。");
                    }
                }
                Page::Countries => {
                    ui.horizontal(|ui| {
                        if ui.button("← 模组").clicked() {
                            self.page = Page::Mods
                        }
                        ui.heading("选择国家")
                    });
                    egui::ScrollArea::vertical()
                        .id_salt("countries")
                        .show(ui, |ui| {
                            egui::Grid::new("country_grid")
                                .num_columns(3)
                                .spacing([12.0, 12.0])
                                .show(ui, |ui| {
                                    for (i, (tag, name)) in
                                        self.countries.clone().iter().enumerate()
                                    {
                                        if ui
                                            .add_sized(
                                                [230.0, 72.0],
                                                egui::Button::new(format!("{name}\n{tag}")),
                                            )
                                            .clicked()
                                        {
                                            self.choose_country(tag.clone())
                                        }
                                        if i % 3 == 2 {
                                            ui.end_row()
                                        }
                                    }
                                });
                        });
                }
                Page::Chapters => {
                    ui.horizontal(|ui| {
                        if ui.button("← 国家").clicked() {
                            self.page = Page::Countries
                        }
                        ui.heading(format!("{} · 国策", self.saved.tag))
                    });
                    if self.focuses.is_empty() {
                        ui.label("此国家尚未解析到国策；查看下方诊断。");
                    }
                    egui::ScrollArea::vertical()
                        .id_salt("focuses")
                        .show(ui, |ui| {
                            for (i, f) in self.focuses.clone().iter().enumerate() {
                                egui::Frame::group(ui.style())
                                    .fill(egui::Color32::from_rgb(247, 242, 250))
                                    .stroke(egui::Stroke::NONE)
                                    .rounding(egui::Rounding::same(16.0))
                                    .inner_margin(16.)
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            if f.image.is_some() {
                                                ui.label("▣");
                                            }
                                            ui.vertical(|ui| {
                                                ui.heading(&f.title);
                                                if !f.desc.is_empty() {
                                                    ui.label(
                                                        f.desc
                                                            .chars()
                                                            .take(170)
                                                            .collect::<String>(),
                                                    );
                                                }
                                            });
                                            if primary_button(ui, "阅读").clicked() {
                                                self.select_focus(i, c)
                                            }
                                        });
                                    });
                            }
                        });
                }
                Page::Reader => {
                    ui.horizontal(|ui| {
                        if ui.button("← 章节").clicked() {
                            self.page = Page::Chapters
                        }
                        ui.label(format!("{} / {}", self.focus_index + 1, self.focuses.len()));
                    });
                    if let Some(f) = self.focuses.get(self.focus_index).cloned() {
                        ui.add_space(16.);
                        ui.heading(&f.title);
                        ui.separator();
                        if let Some(t) = self.texture.clone() {
                            ui.image((t.id(), egui::vec2(160.0, 160.0)));
                        }
                        if f.desc.is_empty() {
                            ui.label("本国策没有找到对应的描述译文。");
                        } else {
                            ui.add(egui::Label::new(&f.desc).wrap());
                        }
                        ui.add_space(24.);
                        ui.horizontal(|ui| {
                            if ui.button("上一个").clicked() && self.focus_index > 0 {
                                self.select_focus(self.focus_index - 1, c)
                            }
                            if ui.button("下一个").clicked()
                                && self.focus_index + 1 < self.focuses.len()
                            {
                                self.select_focus(self.focus_index + 1, c)
                            }
                        });
                        ui.collapsing("解析来源", |ui| {
                            ui.label(format!("{} · {}", f.id, f.source));
                            if f.image.is_none() {
                                ui.label(format!("未找到图片资源：{}", f.icon));
                            }
                        });
                    }
                }
                Page::Settings => {
                    ui.heading("设置");
                    ui.label("自动发现的游戏目录");
                    ui.text_edit_singleline(&mut self.saved.game);
                    ui.label("Steam Workshop / 模组目录");
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.saved.mods_root);
                        if ui.button("浏览").clicked() {
                            if let Some(p) = rfd::FileDialog::new().pick_folder() {
                                self.saved.mods_root = p.display().to_string();
                            }
                        }
                        if ui.button("扫描").clicked() {
                            self.mods = scan_mods(Path::new(&self.saved.mods_root));
                        }
                    });
                    ui.label("选择内容语言");
                    egui::ComboBox::from_id_salt("language")
                        .selected_text(self.saved.language.label())
                        .show_ui(ui, |ui| {
                            for l in [Lang::En, Lang::ZhHans, Lang::ZhHant, Lang::Ja] {
                                if ui
                                    .selectable_value(&mut self.saved.language, l, l.label())
                                    .changed()
                                {
                                    self.reload()
                                }
                            }
                        });
                    if ui.button("重新扫描所选模组").clicked() {
                        self.reload()
                    }
                    if ui.button("保存设置").clicked() {
                        save(&self.saved)
                    }
                    ui.separator();
                    ui.label("不修改游戏原文件。模组未下载时不会自动下载。");
                }
            }
            if !self.status.is_empty() {
                ui.label(&self.status);
            }
            if !self.diagnostics.is_empty() {
                ui.collapsing(
                    format!("诊断 / Diagnostics ({})", self.diagnostics.len()),
                    |ui| {
                        for d in &self.diagnostics {
                            ui.label(d);
                        }
                    },
                );
            }
        });
    }
}
fn main() -> eframe::Result<()> {
    eframe::run_native(
        "HOI4Reader",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1280.0, 840.0])
                .with_min_inner_size([960.0, 640.0]),
            ..Default::default()
        },
        Box::new(|cc| {
            cjk_fonts(&cc.egui_ctx);
            material(&cc.egui_ctx);
            Ok(Box::new(Reader::init()))
        }),
    )
}
