// ═══════════════════════════════════════════════════════════════════════════════
//  Lodestone v3
//  Dark minimalist aesthetic — cool charcoal bg, teal accent (#1eb3a3)
// ═══════════════════════════════════════════════════════════════════════════════

#![allow(dead_code)]

mod bytecode;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use iced::alignment;
use iced::theme::Theme;
use iced::widget::{
    button, column, container, horizontal_rule, pick_list, row, scrollable, text,
    text_input, Space,
};
use iced::{Color, Element, Length, Settings, Size, Task};

// ─────────────────────────────────────────────────────────────────────────────
// Palette — mirrors marsphobos.com CSS custom properties
// ─────────────────────────────────────────────────────────────────────────────

mod pal {
    use iced::Color;

    // Backgrounds — deep cool charcoal
    pub const BG:        Color = Color { r: 0.051, g: 0.059, b: 0.063, a: 1.0 }; // #0d0f10
    pub const BG_RAISED: Color = Color { r: 0.082, g: 0.090, b: 0.094, a: 1.0 }; // #151718
    pub const SURFACE:   Color = Color { r: 0.106, g: 0.118, b: 0.125, a: 1.0 }; // #1b1e20

    // Borders / dividers
    pub const LINE:     Color = Color { r: 0.165, g: 0.184, b: 0.192, a: 1.0 }; // #2a2f31
    pub const LINE_DIM: Color = Color { r: 0.122, g: 0.137, b: 0.145, a: 1.0 }; // #1f2325

    // Text
    pub const INK:   Color = Color { r: 0.918, g: 0.933, b: 0.941, a: 1.0 }; // #eaeff0
    pub const MUTED: Color = Color { r: 0.529, g: 0.565, b: 0.584, a: 1.0 }; // #879095
    pub const FAINT: Color = Color { r: 0.318, g: 0.349, b: 0.365, a: 1.0 }; // #51595d

    // Accent — teal / cyan
    pub const ACCENT:      Color = Color { r: 0.118, g: 0.702, b: 0.639, a: 1.0 }; // #1eb3a3
    pub const ACCENT_DARK: Color = Color { r: 0.082, g: 0.502, b: 0.459, a: 1.0 }; // #158075
    pub const ACCENT_TINT: Color = Color { r: 0.118, g: 0.702, b: 0.639, a: 0.10 };

    // Status
    pub const GREEN:  Color = Color { r: 0.235, g: 0.690, b: 0.431, a: 1.0 }; // #3cb06e
    pub const AMBER:  Color = Color { r: 0.824, g: 0.627, b: 0.235, a: 1.0 }; // #d2a03c
    pub const RED:    Color = Color { r: 0.816, g: 0.333, b: 0.306, a: 1.0 }; // #d0554e
    pub const PURPLE: Color = Color { r: 0.565, g: 0.482, b: 0.839, a: 1.0 }; // #907bd6
}

// ─────────────────────────────────────────────────────────────────────────────
// Domain types
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModLoader {
    Unknown,
    Forge,
    NeoForge,
    Fabric,
    Quilt,
}

impl std::fmt::Display for ModLoader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            ModLoader::Unknown  => "Unknown",
            ModLoader::Forge    => "Forge",
            ModLoader::NeoForge => "NeoForge",
            ModLoader::Fabric   => "Fabric",
            ModLoader::Quilt    => "Quilt",
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Side {
    Unknown,
    Client,
    Server,
    Both,
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Side::Unknown => "Unknown",
            Side::Client  => "Client",
            Side::Server  => "Server",
            Side::Both    => "Both",
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Module (JSON classification file)
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleEntry {
    pub mod_version: String,
    pub mod_tag:     Side,
    pub mod_type:    ModLoader,
}

#[derive(Debug, Deserialize, Serialize)]
struct ModuleHeader {
    module_name:    String,
    module_version: f64,
    module_author:  String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ModuleJson {
    header: ModuleHeader,
    mods:   BTreeMap<String, ModuleEntry>,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name:    String,
    pub version: f64,
    pub author:  String,
    pub mods:    BTreeMap<String, ModuleEntry>,
    pub path:    String,
}

impl Module {
    fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let raw: ModuleJson = serde_json::from_str(&fs::read_to_string(path)?)?;
        Ok(Self {
            name:    raw.header.module_name,
            version: raw.header.module_version,
            author:  raw.header.module_author,
            mods:    raw.mods,
            path:    path.to_string(),
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Jar detection
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct JarInfo {
    pub mod_id:       String,
    pub loader:       ModLoader,
    pub version:      Option<String>,
    pub declared_side:Option<Side>,
}

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub jar_name:     String,
    pub jar_info:     Option<JarInfo>,
    pub parse_error:  Option<String>,
    pub module_entry: Option<ModuleEntry>,
    pub match_quality:MatchQuality,
    /// Side inferred purely from bytecode analysis (no module required)
    pub bytecode_side: Option<crate::bytecode::DetectedSide>,
    pub bytecode_confidence: crate::bytecode::Confidence,
    /// A representative signal string shown in the UI tooltip
    pub bytecode_signal: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchQuality {
    Full,
    Partial,
    Unidentified,
    Unknown,
}

impl ScanResult {
    fn status_label(&self) -> &'static str {
        match self.match_quality {
            MatchQuality::Full         => "Full match",
            MatchQuality::Partial      => "Partial",
            MatchQuality::Unidentified => "Unidentified",
            MatchQuality::Unknown      => "Unknown",
        }
    }
    fn status_color(&self) -> Color {
        match self.match_quality {
            MatchQuality::Full         => pal::GREEN,
            MatchQuality::Partial      => pal::AMBER,
            MatchQuality::Unidentified => pal::RED,
            MatchQuality::Unknown      => pal::FAINT,
        }
    }
    fn effective_side(&self) -> Side {
        // Priority: module entry > manifest declared side > bytecode detection
        if let Some(s) = self.module_entry.as_ref().map(|e| e.mod_tag) {
            return s;
        }
        if let Some(s) = self.jar_info.as_ref().and_then(|i| i.declared_side) {
            return s;
        }
        // Fall back to bytecode evidence
        match &self.bytecode_side {
            Some(crate::bytecode::DetectedSide::Client) => Side::Client,
            Some(crate::bytecode::DetectedSide::Server) => Side::Server,
            Some(crate::bytecode::DetectedSide::Both)   => Side::Both,
            _ => Side::Unknown,
        }
    }

    /// Human-readable source of the side determination
    fn side_source(&self) -> &'static str {
        if self.module_entry.is_some() { return "module"; }
        if self.jar_info.as_ref().and_then(|i| i.declared_side).is_some() { return "manifest"; }
        match self.bytecode_confidence {
            crate::bytecode::Confidence::Annotation     => "annotation",
            crate::bytecode::Confidence::ClassReference => "bytecode",
            crate::bytecode::Confidence::None           => "—",
        }
    }
}

fn read_zip_entry(e: &mut zip::read::ZipFile) -> Result<String, Box<dyn std::error::Error>> {
    let mut s = String::new();
    e.read_to_string(&mut s)?;
    Ok(s)
}

fn toml_str(v: &toml::Value) -> Option<String> {
    v.as_str().map(String::from)
        .or_else(|| v.as_float().map(|f| f.to_string()))
        .or_else(|| v.as_integer().map(|i| i.to_string()))
}

fn json_str(v: &serde_json::Value) -> Option<String> {
    v.as_str().map(String::from)
        .or_else(|| v.as_f64().map(|f| f.to_string()))
}

fn parse_jar(path: &str) -> Result<Option<JarInfo>, Box<dyn std::error::Error>> {
    let mut archive = zip::ZipArchive::new(fs::File::open(path)?)?;
    for i in 0..archive.len() {
        let mut e = archive.by_index(i)?;
        let name = e.name().to_string();
        if name.ends_with("mods.toml")        { return parse_forge(&mut e).map(Some); }
        if name.ends_with("fabric.mod.json")  { return parse_fabric(&mut e).map(Some); }
        if name.ends_with("quilt.mod.json")   { return parse_quilt(&mut e).map(Some); }
        if name.ends_with("mcmod.info")       { return parse_legacy(&mut e).map(Some); }
    }
    Ok(None)
}

fn parse_forge(e: &mut zip::read::ZipFile) -> Result<JarInfo, Box<dyn std::error::Error>> {
    let raw = read_zip_entry(e)?;
    let lower = raw.to_lowercase();
    let loader = if lower.contains("neoforge") || lower.contains("neo-forge") {
        ModLoader::NeoForge
    } else {
        ModLoader::Forge
    };
    let parsed: toml::Value = toml::from_str(&raw)?;
    let first = parsed.get("mods").and_then(|v| v.as_array()).and_then(|a| a.first());
    let mod_id  = first.and_then(|m| m.get("modId")).and_then(|v| v.as_str())
                       .map(String::from).unwrap_or_else(|| "unknown".into());
    let version = first.and_then(|m| m.get("version").or_else(|| m.get("modVersion")))
                       .and_then(toml_str);
    Ok(JarInfo { mod_id, loader, version, declared_side: None })
}

fn parse_fabric(e: &mut zip::read::ZipFile) -> Result<JarInfo, Box<dyn std::error::Error>> {
    let v: serde_json::Value = serde_json::from_str(&read_zip_entry(e)?)?;
    let mod_id  = v.get("id").and_then(|x| x.as_str()).map(String::from)
                   .unwrap_or_else(|| "unknown".into());
    let version = v.get("version").and_then(json_str);
    let declared_side = v.get("environment").and_then(|x| x.as_str()).and_then(|s| match s {
        "client" => Some(Side::Client),
        "server" => Some(Side::Server),
        "*"      => Some(Side::Both),
        _        => None,
    });
    Ok(JarInfo { mod_id, loader: ModLoader::Fabric, version, declared_side })
}

fn parse_quilt(e: &mut zip::read::ZipFile) -> Result<JarInfo, Box<dyn std::error::Error>> {
    let v: serde_json::Value = serde_json::from_str(&read_zip_entry(e)?)?;
    let ql = v.get("quilt_loader");
    let mod_id  = ql.and_then(|l| l.get("id")).and_then(|x| x.as_str())
                    .map(String::from).unwrap_or_else(|| "unknown".into());
    let version = ql.and_then(|l| l.get("version")).and_then(json_str);
    Ok(JarInfo { mod_id, loader: ModLoader::Quilt, version, declared_side: None })
}

fn parse_legacy(e: &mut zip::read::ZipFile) -> Result<JarInfo, Box<dyn std::error::Error>> {
    let v: serde_json::Value = serde_json::from_str(&read_zip_entry(e)?)?;
    let first = v.as_array().and_then(|a| a.first());
    let mod_id  = first.and_then(|m| m.get("modid")).and_then(|x| x.as_str())
                       .map(String::from).unwrap_or_else(|| "unknown".into());
    let version = first.and_then(|m| m.get("version")).and_then(json_str);
    Ok(JarInfo { mod_id, loader: ModLoader::Forge, version, declared_side: None })
}

// ─────────────────────────────────────────────────────────────────────────────
// Scan
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct ScanSummary {
    pub total: usize,
    pub full:  usize,
    pub partial: usize,
    pub unidentified: usize,
    pub unknown: usize,
}

fn scan_directory(dir: &str, module: &Module) -> (Vec<ScanResult>, ScanSummary) {
    let mut jars: Vec<String> = fs::read_dir(dir)
        .map(|rd| rd
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("jar"))
            .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .collect()
        )
        .unwrap_or_default();
    jars.sort();

    let mut results = Vec::new();
    for jar_name in jars {
        let path = format!("{}/{}", dir.trim_end_matches('/'), jar_name);

        let (jar_info, parse_error) = match parse_jar(&path) {
            Ok(i)  => (i, None),
            Err(e) => (None, Some(e.to_string())),
        };

        // Bytecode analysis — runs regardless of whether a module is loaded
        let bc = bytecode::analyse_jar(&path).unwrap_or_else(bytecode::BytecodeEvidence::unknown);

        let (module_entry, match_quality) = if let Some(info) = &jar_info {
            if let Some(entry) = module.mods.get(&info.mod_id).cloned() {
                let version_ok = entry.mod_version == "*"
                    || info.version.as_deref().map(|v| v == entry.mod_version).unwrap_or(false);
                let loader_ok  = info.loader == entry.mod_type;
                let q = if version_ok && loader_ok { MatchQuality::Full } else { MatchQuality::Partial };
                (Some(entry), q)
            } else {
                (None, MatchQuality::Unidentified)
            }
        } else {
            (None, MatchQuality::Unknown)
        };

        results.push(ScanResult {
            jar_name, jar_info, parse_error, module_entry, match_quality,
            bytecode_side:       Some(bc.side),
            bytecode_confidence: bc.confidence,
            bytecode_signal:     bc.signal,
        });
    }

    let summary = ScanSummary {
        total:        results.len(),
        full:         results.iter().filter(|r| r.match_quality == MatchQuality::Full).count(),
        partial:      results.iter().filter(|r| r.match_quality == MatchQuality::Partial).count(),
        unidentified: results.iter().filter(|r| r.match_quality == MatchQuality::Unidentified).count(),
        unknown:      results.iter().filter(|r| r.match_quality == MatchQuality::Unknown).count(),
    };
    (results, summary)
}

// ─────────────────────────────────────────────────────────────────────────────
// Operations
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation { Zip, Move, Delete, Export }

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Operation::Zip    => "Zip",
            Operation::Move   => "Move",
            Operation::Delete => "Delete",
            Operation::Export => "Export list",
        })
    }
}

fn run_operation(
    op: Operation, dir: &str,
    results: &[ScanResult], filter_side: Side, output: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    let targets: Vec<&ScanResult> = results.iter()
        .filter(|r| r.effective_side() == filter_side).collect();

    match op {
        Operation::Zip => {
            use zip::write::FileOptions;
            let mut w = zip::ZipWriter::new(fs::File::create(output)?);
            let opts = FileOptions::default();
            let mut n = 0usize;
            for r in &targets {
                let src = Path::new(dir).join(&r.jar_name);
                if src.is_file() {
                    let mut buf = Vec::new();
                    fs::File::open(&src)?.read_to_end(&mut buf)?;
                    w.start_file(&r.jar_name, opts)?;
                    w.write_all(&buf)?;
                    n += 1;
                }
            }
            w.finish()?;
            Ok(n)
        }
        Operation::Move => {
            fs::create_dir_all(output)?;
            let mut n = 0usize;
            for r in &targets {
                let src = Path::new(dir).join(&r.jar_name);
                let dst = Path::new(output).join(&r.jar_name);
                if src.is_file() {
                    if fs::rename(&src, &dst).is_err() {
                        fs::copy(&src, &dst)?;
                        fs::remove_file(&src)?;
                    }
                    n += 1;
                }
            }
            Ok(n)
        }
        Operation::Delete => {
            let mut n = 0usize;
            for r in &targets {
                let p = Path::new(dir).join(&r.jar_name);
                if p.is_file() { fs::remove_file(p)?; n += 1; }
            }
            Ok(n)
        }
        Operation::Export => {
            let mut f = fs::File::create(output)?;
            let mut n = 0usize;
            for r in &targets { writeln!(f, "{}", r.jar_name)?; n += 1; }
            Ok(n)
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Module discovery
// ─────────────────────────────────────────────────────────────────────────────

fn discover_modules() -> Vec<String> {
    let exe_base = std::env::current_exe()
        .ok().and_then(|p| p.parent().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."));

    let mut found = Vec::new();
    for tp in &[exe_base.join("test.json"), PathBuf::from("test.json")] {
        if tp.exists() { found.push(tp.display().to_string()); break; }
    }
    for dir in &[exe_base.join("modules"), PathBuf::from("modules")] {
        if let Ok(rd) = fs::read_dir(dir) {
            for e in rd.filter_map(Result::ok) {
                let p = e.path();
                if p.extension().and_then(|s| s.to_str()) == Some("json") {
                    found.push(p.display().to_string());
                }
            }
            break;
        }
    }
    found
}

// ─────────────────────────────────────────────────────────────────────────────
// App state
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Panel { Scan, Operate }

struct App {
    modules:         Vec<String>,
    selected_module: Option<String>,
    loaded_module:   Option<Module>,
    directory:       String,
    scan_results:    Vec<ScanResult>,
    summary:         ScanSummary,
    op_side:         Side,
    op:              Operation,
    op_output:       String,
    op_confirm:      String,
    active_panel:    Panel,
    filter_side:     Option<Side>,
    log:             Vec<(String, LogLevel)>,
}

#[derive(Debug, Clone, Copy)]
enum LogLevel { Info, Ok, Warn, Err }

impl Default for App {
    fn default() -> Self {
        let modules = discover_modules();
        let sel = modules.first().cloned();
        Self {
            modules, selected_module: sel, loaded_module: None,
            directory: String::new(),
            scan_results: Vec::new(), summary: ScanSummary::default(),
            op_side: Side::Client, op: Operation::Zip,
            op_output: String::new(), op_confirm: String::new(),
            active_panel: Panel::Scan, filter_side: None,
            log: vec![("Lodestone ready.".into(), LogLevel::Info)],
        }
    }
}

impl App {
    fn push_log(&mut self, msg: impl Into<String>, level: LogLevel) {
        let s = msg.into();
        if self.log.last().map(|(t, _)| t == &s).unwrap_or(false) { return; }
        self.log.push((s, level));
        if self.log.len() > 200 { self.log.drain(0..self.log.len() - 200); }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Messages
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
enum Msg {
    NavPanel(Panel),
    RefreshModules,
    ModuleSelected(String),
    LoadModule,
    DirChanged(String),
    BrowseDir,
    DirPicked(Option<PathBuf>),
    ScanDir,
    FilterSide(Option<Side>),
    OpSideSelected(Side),
    OpSelected(Operation),
    OpOutputChanged(String),
    OpConfirmChanged(String),
    RunOp,
}

// ─────────────────────────────────────────────────────────────────────────────
// Update
// ─────────────────────────────────────────────────────────────────────────────

fn update(app: &mut App, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::NavPanel(p) => app.active_panel = p,

        Msg::RefreshModules => {
            app.modules = discover_modules();
            if app.selected_module.as_ref().map(|s| !app.modules.contains(s)).unwrap_or(true) {
                app.selected_module = app.modules.first().cloned();
            }
            app.push_log(format!("{} module(s) found.", app.modules.len()), LogLevel::Info);
        }

        Msg::ModuleSelected(p) => app.selected_module = Some(p),

        Msg::LoadModule => match app.selected_module.as_deref() {
            None => app.push_log("Select a module first.", LogLevel::Warn),
            Some(path) => match Module::load(path) {
                Ok(m) => {
                    let msg = format!("'{}' — {} entries.", m.name, m.mods.len());
                    app.scan_results.clear();
                    app.summary = ScanSummary::default();
                    app.loaded_module = Some(m);
                    app.push_log(msg, LogLevel::Ok);
                }
                Err(e) => app.push_log(format!("Load failed: {e}"), LogLevel::Err),
            },
        },

        Msg::DirChanged(v) => app.directory = v,

        Msg::BrowseDir => return Task::perform(
            async { rfd::AsyncFileDialog::new().pick_folder().await },
            |h| Msg::DirPicked(h.map(|x| x.path().to_path_buf())),
        ),

        Msg::DirPicked(p) => if let Some(p) = p {
            app.directory = p.display().to_string();
        },

        Msg::ScanDir => {
            let Some(module) = &app.loaded_module else {
                app.push_log("Load a module first.", LogLevel::Warn);
                return Task::none();
            };
            let dir = app.directory.trim().to_string();
            if dir.is_empty() {
                app.push_log("Choose a mods directory first.", LogLevel::Warn);
                return Task::none();
            }
            let (results, summary) = scan_directory(&dir, module);
            let msg = format!(
                "{} jars — {} full, {} partial, {} unidentified.",
                summary.total, summary.full, summary.partial, summary.unidentified
            );
            app.scan_results = results;
            app.summary = summary;
            app.push_log(msg, LogLevel::Ok);
        }

        Msg::FilterSide(s) => app.filter_side = s,

        Msg::OpSideSelected(s) => app.op_side = s,
        Msg::OpSelected(o) => { app.op = o; app.op_output.clear(); app.op_confirm.clear(); }
        Msg::OpOutputChanged(v) => app.op_output = v,
        Msg::OpConfirmChanged(v) => app.op_confirm = v,

        Msg::RunOp => {
            if app.loaded_module.is_none() {
                app.push_log("Load a module first.", LogLevel::Warn);
                return Task::none();
            }
            if app.scan_results.is_empty() {
                app.push_log("Scan a directory first.", LogLevel::Warn);
                return Task::none();
            }
            if app.op == Operation::Delete && app.op_confirm.trim() != "DELETE" {
                app.push_log("Type DELETE to confirm deletion.", LogLevel::Warn);
                return Task::none();
            }
            let output = app.op_output.trim().to_string();
            if app.op != Operation::Delete && output.is_empty() {
                app.push_log("Enter an output path.", LogLevel::Warn);
                return Task::none();
            }
            let dir = app.directory.trim().to_string();
            match run_operation(app.op, &dir, &app.scan_results, app.op_side, &output) {
                Ok(n)  => app.push_log(format!("{n} file(s) affected."), LogLevel::Ok),
                Err(e) => app.push_log(format!("Error: {e}"), LogLevel::Err),
            }
        }
    }
    Task::none()
}

// ─────────────────────────────────────────────────────────────────────────────
// Style helpers
// ─────────────────────────────────────────────────────────────────────────────

fn tc(c: Color) -> impl Fn(&Theme) -> iced::widget::text::Style {
    move |_| iced::widget::text::Style { color: Some(c) }
}

fn card_container(content: Element<'_, Msg>) -> Element<'_, Msg> {
    container(content)
        .style(|_| container::Style {
            background: Some(pal::BG_RAISED.into()),
            border: iced::border::Border {
                color:  pal::LINE,
                width:  1.0,
                radius: 12.0.into(),
            },
            shadow: iced::Shadow {
                color:       Color { r: 0.0, g: 0.0, b: 0.0, a: 0.25 },
                offset:      iced::Vector::new(0.0, 2.0),
                blur_radius: 12.0,
            },
            ..Default::default()
        })
        .padding(20)
        .into()
}

// Primary CTA — solid teal pill
fn btn_primary<'a>(label: &'a str) -> button::Button<'a, Msg> {
    button(
        text(label).size(13).style(tc(Color::WHITE))
    )
    .style(|_, status| {
        let bg = match status {
            button::Status::Hovered  => pal::ACCENT_DARK,
            button::Status::Pressed  => Color { a: 0.80, ..pal::ACCENT_DARK },
            button::Status::Disabled => Color { a: 0.35, ..pal::ACCENT },
            button::Status::Active   => pal::ACCENT,
        };
        button::Style {
            background: Some(bg.into()),
            text_color: Color::WHITE,
            border: iced::border::Border {
                color:  Color::TRANSPARENT,
                width:  0.0,
                radius: 8.0.into(),
            },
            ..Default::default()
        }
    })
    .padding([9, 18])
}

// Danger pill — muted red
fn btn_danger<'a>(label: &'a str) -> button::Button<'a, Msg> {
    button(text(label).size(13).style(tc(Color::WHITE)))
        .style(|_, status| {
            let bg = match status {
                button::Status::Hovered  => Color { r: 0.700, g: 0.267, b: 0.247, a: 1.0 },
                button::Status::Pressed  => Color { a: 0.80, ..pal::RED },
                button::Status::Disabled => Color { a: 0.35, ..pal::RED },
                button::Status::Active   => pal::RED,
            };
            button::Style {
                background: Some(bg.into()),
                text_color: Color::WHITE,
                border: iced::border::Border { radius: 8.0.into(), width: 0.0, color: Color::TRANSPARENT },
                ..Default::default()
            }
        })
        .padding([9, 18])
}

// Ghost — subtle surface button
fn btn_ghost<'a>(label: &'a str) -> button::Button<'a, Msg> {
    button(text(label).size(13).style(tc(pal::MUTED)))
        .style(|_, status| {
            let bg = match status {
                button::Status::Hovered => pal::SURFACE,
                button::Status::Pressed => pal::LINE_DIM,
                _ => Color::TRANSPARENT,
            };
            button::Style {
                background: Some(bg.into()),
                text_color: pal::MUTED,
                border: iced::border::Border {
                    color: pal::LINE,
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            }
        })
        .padding([9, 16])
}

// Nav tab button
fn btn_nav<'a>(label: &'a str, active: bool, msg: Msg) -> Element<'a, Msg> {
    let fg = if active { pal::INK } else { pal::FAINT };
    let bg = if active { pal::SURFACE } else { Color::TRANSPARENT };
    let bdr_color = if active { pal::LINE } else { Color::TRANSPARENT };
    button(text(label).size(13).style(tc(fg)))
        .style(move |_, status| {
            let bg = match status {
                button::Status::Hovered if !active => Color { a: 0.5, ..pal::SURFACE },
                _ => bg,
            };
            button::Style {
                background: Some(bg.into()),
                text_color: fg,
                border: iced::border::Border { color: bdr_color, width: 1.0, radius: 8.0.into() },
                ..Default::default()
            }
        })
        .on_press(msg)
        .padding([7, 16])
        .into()
}

// Filter chip
fn filter_chip<'a>(label: &'a str, active: bool, msg: Msg) -> Element<'a, Msg> {
    let (bg, fg, bdr) = if active {
        (pal::ACCENT_TINT, pal::ACCENT, pal::ACCENT)
    } else {
        (Color::TRANSPARENT, pal::FAINT, pal::LINE)
    };
    button(text(label).size(12).style(tc(fg)))
        .style(move |_, status| {
            let bg = match status {
                button::Status::Hovered if !active => Color { a: 0.5, ..pal::SURFACE },
                _ => bg,
            };
            button::Style {
                background: Some(bg.into()),
                text_color: fg,
                border: iced::border::Border { color: bdr, width: 1.0, radius: 6.0.into() },
                ..Default::default()
            }
        })
        .on_press(msg)
        .padding([5, 12])
        .into()
}

fn eyebrow<'a>(label: &'a str) -> Element<'a, Msg> {
    text(label)
        .size(10)
        .style(tc(pal::FAINT))
        .into()
}

fn input_style_base() -> iced::widget::text_input::Style {
    iced::widget::text_input::Style {
        background:  pal::BG.into(),
        border: iced::border::Border { color: pal::LINE, width: 1.0, radius: 8.0.into() },
        icon:        pal::FAINT,
        placeholder: pal::FAINT,
        value:       pal::INK,
        selection:   Color { a: 0.20, ..pal::ACCENT },
    }
}

fn input_style_danger() -> iced::widget::text_input::Style {
    iced::widget::text_input::Style {
        border: iced::border::Border { color: pal::RED, width: 1.0, radius: 8.0.into() },
        ..input_style_base()
    }
}

fn pick_style() -> iced::widget::pick_list::Style {
    iced::widget::pick_list::Style {
        text_color:        pal::INK,
        placeholder_color: pal::FAINT,
        handle_color:      pal::MUTED,
        background:        pal::BG.into(),
        border: iced::border::Border { color: pal::LINE, width: 1.0, radius: 8.0.into() },
    }
}

fn divider<'a>() -> Element<'a, Msg> {
    container(horizontal_rule(1))
        .style(|_| container::Style { ..Default::default() })
        .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// View — top bar
// ─────────────────────────────────────────────────────────────────────────────

fn view_topbar(app: &App) -> Element<'_, Msg> {
    let pill_text: String = if app.scan_results.is_empty() {
        "No scan yet".to_string()
    } else {
        format!(
            "{} jars  ·  {} matched  ·  {} unidentified",
            app.summary.total,
            app.summary.full + app.summary.partial,
            app.summary.unidentified
        )
    };

    let summary_pill = container(
        text(pill_text).size(11).style(tc(pal::FAINT))
    )
    .style(|_| container::Style {
        background: Some(pal::SURFACE.into()),
        border: iced::border::Border { color: pal::LINE, width: 1.0, radius: 6.0.into() },
        ..Default::default()
    })
    .padding([5, 12]);

    let nav = row![
        btn_nav("Scan",    app.active_panel == Panel::Scan,    Msg::NavPanel(Panel::Scan)),
        btn_nav("Operate", app.active_panel == Panel::Operate, Msg::NavPanel(Panel::Operate)),
    ]
    .spacing(4);

    container(
        row![
            text("lodestone").size(15).style(tc(pal::INK)),
            Space::with_width(20),
            nav,
            Space::with_width(Length::Fill),
            summary_pill,
        ]
        .align_y(alignment::Vertical::Center)
        .spacing(0),
    )
    .padding([10, 20])
    .style(|_| container::Style {
        background: Some(pal::BG_RAISED.into()),
        ..Default::default()
    })
    .width(Length::Fill)
    .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// View — log strip (bottom)
// ─────────────────────────────────────────────────────────────────────────────

fn view_log(app: &App) -> Element<'_, Msg> {
    let (msg_text, color): (String, Color) = match app.log.last() {
        None => ("—".to_string(), pal::FAINT),
        Some((t, lv)) => {
            let c = match lv {
                LogLevel::Ok   => pal::GREEN,
                LogLevel::Warn => pal::AMBER,
                LogLevel::Err  => pal::RED,
                LogLevel::Info => pal::MUTED,
            };
            (t.clone(), c)
        }
    };

    container(
        row![
            text("●").size(8).style(tc(color)),
            Space::with_width(8),
            text(msg_text).size(11).style(tc(pal::FAINT)),
        ]
        .align_y(alignment::Vertical::Center),
    )
    .padding([7, 20])
    .style(|_| container::Style {
        background: Some(pal::BG_RAISED.into()),
        ..Default::default()
    })
    .width(Length::Fill)
    .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// View — Scan panel
// ─────────────────────────────────────────────────────────────────────────────

fn view_scan(app: &App) -> Element<'_, Msg> {

    // ── Left column: controls ─────────────────────────────────────────────

    let module_loaded_info: Element<'_, Msg> = if let Some(m) = &app.loaded_module {
        column![
            text(&m.name).size(13).style(tc(pal::INK)),
            text(format!("v{}  ·  {}  ·  {} entries", m.version, m.author, m.mods.len()))
                .size(11).style(tc(pal::MUTED)),
        ]
        .spacing(2)
        .into()
    } else {
        text("No module loaded").size(12).style(tc(pal::FAINT)).into()
    };

    let module_card = card_container(
        column![
            eyebrow("MODULE"),
            Space::with_height(10),
            pick_list(
                app.modules.clone(),
                app.selected_module.clone(),
                Msg::ModuleSelected,
            )
            .placeholder("Select a module file…")
            .style(|_, _| pick_style())
            .width(Length::Fill),
            Space::with_height(10),
            row![
                btn_ghost("Refresh").on_press(Msg::RefreshModules),
                btn_primary("Load module").on_press(Msg::LoadModule),
            ]
            .spacing(8),
            Space::with_height(12),
            divider(),
            Space::with_height(12),
            module_loaded_info,
        ]
        .spacing(0)
        .into(),
    );

    let dir_card = card_container(
        column![
            eyebrow("MODS DIRECTORY"),
            Space::with_height(10),
            row![
                text_input("Path to mods folder…", &app.directory)
                    .on_input(Msg::DirChanged)
                    .style(|_, _| input_style_base())
                    .padding([9, 12])
                    .size(13),
                btn_ghost("Browse").on_press(Msg::BrowseDir),
            ]
            .spacing(8)
            .align_y(alignment::Vertical::Center),
            Space::with_height(10),
            btn_primary("Scan directory").on_press(Msg::ScanDir),
        ]
        .spacing(0)
        .into(),
    );

    let left_col = column![module_card, Space::with_height(12), dir_card]
        .spacing(0)
        .width(280);

    // ── Right column: results ─────────────────────────────────────────────

    let filter_row = row![
        filter_chip("All",     app.filter_side.is_none(),                      Msg::FilterSide(None)),
        filter_chip("Client",  app.filter_side == Some(Side::Client),           Msg::FilterSide(Some(Side::Client))),
        filter_chip("Server",  app.filter_side == Some(Side::Server),           Msg::FilterSide(Some(Side::Server))),
        filter_chip("Both",    app.filter_side == Some(Side::Both),             Msg::FilterSide(Some(Side::Both))),
        filter_chip("Unknown", app.filter_side == Some(Side::Unknown),          Msg::FilterSide(Some(Side::Unknown))),
    ]
    .spacing(6);

    let filtered: Vec<&ScanResult> = app.scan_results.iter()
        .filter(|r| app.filter_side.map(|s| r.effective_side() == s).unwrap_or(true))
        .collect();

    let results_body: Element<'_, Msg> = if app.scan_results.is_empty() {
        let step = |n: &'static str, title: &'static str, desc: &'static str| -> Element<'_, Msg> {
            row![
                container(
                    text(n).size(11).style(tc(pal::ACCENT))
                )
                .style(|_| container::Style {
                    background: Some(pal::ACCENT_TINT.into()),
                    border: iced::border::Border { color: pal::ACCENT, width: 1.0, radius: 6.0.into() },
                    ..Default::default()
                })
                .padding([4, 10]),
                Space::with_width(14),
                column![
                    text(title).size(13).style(tc(pal::INK)),
                    text(desc).size(11).style(tc(pal::FAINT)),
                ].spacing(2),
            ]
            .align_y(alignment::Vertical::Center)
            .into()
        };

        container(
            column![
                text("Get started").size(18).style(tc(pal::INK)),
                Space::with_height(4),
                text("Follow these steps to classify your mods.")
                    .size(12).style(tc(pal::MUTED)),
                Space::with_height(28),
                step("01", "Load a module", "Select and load a classification database from the left panel."),
                Space::with_height(16),
                step("02", "Choose a directory", "Point Lodestone at your Minecraft mods folder."),
                Space::with_height(16),
                step("03", "Scan", "Lodestone will classify every jar by side, loader, and source."),
            ]
            .spacing(0),
        )
        .padding([36, 40])
        .style(|_| container::Style {
            background: Some(pal::BG_RAISED.into()),
            border: iced::border::Border { color: pal::LINE, width: 1.0, radius: 12.0.into() },
            ..Default::default()
        })
        .width(Length::Fill)
        .into()
    } else {
        // Table header
        let tbl_header = container(
            row![
                text("FILE").size(10).style(tc(pal::FAINT)).width(Length::FillPortion(5)),
                text("MOD ID").size(10).style(tc(pal::FAINT)).width(Length::FillPortion(3)),
                text("LOADER").size(10).style(tc(pal::FAINT)).width(Length::FillPortion(2)),
                text("VERSION").size(10).style(tc(pal::FAINT)).width(Length::FillPortion(2)),
                text("SIDE").size(10).style(tc(pal::FAINT)).width(Length::FillPortion(2)),
                text("SOURCE").size(10).style(tc(pal::FAINT)).width(Length::FillPortion(2)),
                text("MATCH").size(10).style(tc(pal::FAINT)).width(Length::FillPortion(2)),
            ]
            .spacing(10),
        )
        .padding([8, 14])
        .style(|_| container::Style {
            background: Some(pal::SURFACE.into()),
            border: iced::border::Border {
                color: pal::LINE,
                width: 1.0,
                radius: iced::border::Radius { top_left: 10.0, top_right: 10.0, bottom_right: 0.0, bottom_left: 0.0 },
            },
            ..Default::default()
        });

        let mut rows: Vec<Element<'_, Msg>> = Vec::new();
        for (i, r) in filtered.iter().enumerate() {
            let bg = if i % 2 == 0 { pal::BG } else { pal::BG_RAISED };
            let mod_id  = r.jar_info.as_ref().map(|j| j.mod_id.as_str()).unwrap_or("—");
            let loader  = r.jar_info.as_ref().map(|j| j.loader).unwrap_or(ModLoader::Unknown);
            let version = r.jar_info.as_ref().and_then(|j| j.version.as_deref()).unwrap_or("—");
            let side    = r.effective_side();

            let loader_color = match loader {
                ModLoader::Fabric   => pal::ACCENT,
                ModLoader::Quilt    => pal::PURPLE,
                ModLoader::Forge    | ModLoader::NeoForge => pal::AMBER,
                ModLoader::Unknown  => pal::FAINT,
            };
            let side_color = match side {
                Side::Client  => pal::ACCENT,
                Side::Server  => pal::GREEN,
                Side::Both    => pal::PURPLE,
                Side::Unknown => pal::FAINT,
            };

            let is_last = i == filtered.len() - 1;
            let radius: iced::border::Radius = if is_last {
                iced::border::Radius { top_left: 0.0, top_right: 0.0, bottom_right: 10.0, bottom_left: 10.0 }
            } else {
                0.0.into()
            };

            let source = r.side_source();
            let source_color = match source {
                "module"      => pal::INK,
                "manifest"    => pal::ACCENT,
                "annotation"  => pal::GREEN,
                "bytecode"    => pal::PURPLE,
                _             => pal::FAINT,
            };

            rows.push(
                container(
                    row![
                        text(&r.jar_name).size(12).style(tc(pal::INK))
                            .width(Length::FillPortion(5)),
                        text(mod_id).size(12).style(tc(pal::MUTED))
                            .width(Length::FillPortion(3)),
                        text(loader.to_string()).size(12).style(tc(loader_color))
                            .width(Length::FillPortion(2)),
                        text(version).size(12).style(tc(pal::MUTED))
                            .width(Length::FillPortion(2)),
                        text(side.to_string()).size(12).style(tc(side_color))
                            .width(Length::FillPortion(2)),
                        text(source).size(12).style(tc(source_color))
                            .width(Length::FillPortion(2)),
                        text(r.status_label()).size(12).style(tc(r.status_color()))
                            .width(Length::FillPortion(2)),
                    ]
                    .spacing(10)
                    .align_y(alignment::Vertical::Center),
                )
                .padding([9, 14])
                .style(move |_| container::Style {
                    background: Some(bg.into()),
                    border: iced::border::Border {
                        color: pal::LINE_DIM,
                        width: 1.0,
                        radius,
                    },
                    ..Default::default()
                })
                .into(),
            );
        }

        column![
            tbl_header,
            column(rows).spacing(0),
        ]
        .spacing(0)
        .into()
    };

    let right_col = column![
        filter_row,
        Space::with_height(10),
        results_body,
    ]
    .spacing(0)
    .width(Length::Fill);

    row![
        left_col,
        Space::with_width(20),
        right_col,
    ]
    .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// View — Operate panel
// ─────────────────────────────────────────────────────────────────────────────

fn view_operate(app: &App) -> Element<'_, Msg> {
    let affected = app.scan_results.iter()
        .filter(|r| r.effective_side() == app.op_side).count();

    let op_card = card_container(column![
        eyebrow("ACTION"),
        Space::with_height(10),
        row![
            column![
                eyebrow("TARGET SIDE"),
                Space::with_height(6),
                pick_list(
                    vec![Side::Client, Side::Server, Side::Both, Side::Unknown],
                    Some(app.op_side), Msg::OpSideSelected,
                )
                .style(|_, _| pick_style()).width(Length::Fill),
            ].spacing(0).width(Length::FillPortion(1)),
            column![
                eyebrow("OPERATION"),
                Space::with_height(6),
                pick_list(
                    vec![Operation::Zip, Operation::Move, Operation::Delete, Operation::Export],
                    Some(app.op), Msg::OpSelected,
                )
                .style(|_, _| pick_style()).width(Length::Fill),
            ].spacing(0).width(Length::FillPortion(1)),
        ].spacing(12),
    ].spacing(0).into());

    let output_card: Element<'_, Msg> = if app.op == Operation::Delete {
        card_container(column![
            eyebrow("CONFIRMATION REQUIRED"),
            Space::with_height(6),
            text("This permanently deletes matching files. Type DELETE below to confirm.")
                .size(12).style(tc(pal::AMBER)),
            Space::with_height(8),
            text_input("Type DELETE to confirm…", &app.op_confirm)
                .on_input(Msg::OpConfirmChanged)
                .style(|_, _| input_style_danger())
                .padding([9, 12]).size(13),
        ].spacing(0).into())
    } else {
        let placeholder = match app.op {
            Operation::Zip    => "Output .zip file path",
            Operation::Move   => "Destination directory",
            Operation::Export => "Output .txt file path",
            Operation::Delete => unreachable!(),
        };
        card_container(column![
            eyebrow("OUTPUT PATH"),
            Space::with_height(8),
            text_input(placeholder, &app.op_output)
                .on_input(Msg::OpOutputChanged)
                .style(|_, _| input_style_base())
                .padding([9, 12]).size(13),
        ].spacing(0).into())
    };

    // Preview strip
    let preview = container(
        row![
            text(affected.to_string()).size(28).style(tc(pal::ACCENT)),
            Space::with_width(12),
            column![
                text("files will be affected").size(12).style(tc(pal::INK)),
                text(format!("side: {}", app.op_side))
                    .size(11).style(tc(pal::MUTED)),
            ].spacing(3),
        ]
        .align_y(alignment::Vertical::Center),
    )
    .style(|_| container::Style {
        background: Some(pal::BG_RAISED.into()),
        border: iced::border::Border {
            color: pal::LINE,
            width: 1.0,
            radius: 10.0.into(),
        },
        ..Default::default()
    })
    .padding([16, 20])
    .width(Length::Fill);

    let run_btn: Element<'_, Msg> = if app.op == Operation::Delete {
        btn_danger("Delete files").on_press(Msg::RunOp).into()
    } else {
        btn_primary(match app.op {
            Operation::Zip    => "Create zip",
            Operation::Move   => "Move files",
            Operation::Export => "Export list",
            Operation::Delete => unreachable!(),
        })
        .on_press(Msg::RunOp)
        .into()
    };

    column![
        op_card,
        Space::with_height(12),
        output_card,
        Space::with_height(12),
        preview,
        Space::with_height(16),
        run_btn,
    ]
    .spacing(0)
    .width(500)
    .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// Root view
// ─────────────────────────────────────────────────────────────────────────────

fn view(app: &App) -> Element<'_, Msg> {
    let panel: Element<'_, Msg> = match app.active_panel {
        Panel::Scan    => view_scan(app),
        Panel::Operate => view_operate(app),
    };

    let main_content = container(
        scrollable(
            container(panel)
                .padding([28, 32])
                .width(Length::Fill)
        )
        .height(Length::Fill),
    )
    .style(|_| container::Style {
        background: Some(pal::BG.into()),
        ..Default::default()
    })
    .width(Length::Fill)
    .height(Length::Fill);

    let top_divider = container(Space::with_height(1))
        .style(|_| container::Style {
            background: Some(pal::LINE.into()),
            ..Default::default()
        })
        .width(Length::Fill);

    let bot_divider = container(Space::with_height(1))
        .style(|_| container::Style {
            background: Some(pal::LINE_DIM.into()),
            ..Default::default()
        })
        .width(Length::Fill);

    container(
        column![
            view_topbar(app),
            top_divider,
            main_content,
            bot_divider,
            view_log(app),
        ]
    )
    .style(|_| container::Style {
        background: Some(pal::BG.into()),
        ..Default::default()
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// Entry point
// ─────────────────────────────────────────────────────────────────────────────

fn main() -> iced::Result {
    iced::application("Lodestone", update, view)
        .theme(|_| Theme::Dark)
        .window(iced::window::Settings {
            size:     Size::new(1280.0, 800.0),
            min_size: Some(Size::new(980.0, 640.0)),
            ..Default::default()
        })
        .settings(Settings { antialiasing: true, ..Default::default() })
        .run_with(|| (App::default(), Task::none()))
}
