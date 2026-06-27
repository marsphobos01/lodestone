// ─────────────────────────────────────────────────────────────────────────────
// App state, messages, and the update function
// ─────────────────────────────────────────────────────────────────────────────

use std::path::PathBuf;

use iced::Task;

use crate::domain::Side;
use crate::module::{discover_modules, Module};
use crate::operations::{run_operation, Operation};
use crate::scanner::{scan_directory, ScanResult, ScanSummary};

// ── Navigation panels ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Panel {
    Scan,
    Operate,
}

// ── Log level ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Info,
    Ok,
    Warn,
    Err,
}

// ── Application state ─────────────────────────────────────────────────────────

pub struct App {
    pub modules: Vec<String>,
    pub selected_module: Option<String>,
    pub loaded_module: Option<Module>,
    pub directory: String,
    pub scan_results: Vec<ScanResult>,
    pub summary: ScanSummary,
    pub op_side: Side,
    pub op: Operation,
    pub op_output: String,
    pub op_confirm: String,
    pub active_panel: Panel,
    pub filter_side: Option<Side>,
    pub log: Vec<(String, LogLevel)>,
}

impl Default for App {
    fn default() -> Self {
        let modules = discover_modules();
        let sel = modules.first().cloned();
        Self {
            modules,
            selected_module: sel,
            loaded_module: None,
            directory: String::new(),
            scan_results: Vec::new(),
            summary: ScanSummary::default(),
            op_side: Side::Client,
            op: Operation::Zip,
            op_output: String::new(),
            op_confirm: String::new(),
            active_panel: Panel::Scan,
            filter_side: None,
            log: vec![("Lodestone ready.".into(), LogLevel::Info)],
        }
    }
}

impl App {
    pub fn push_log(&mut self, msg: impl Into<String>, level: LogLevel) {
        let s = msg.into();
        if self.log.last().map(|(t, _)| t == &s).unwrap_or(false) {
            return;
        }
        self.log.push((s, level));
        if self.log.len() > 200 {
            self.log.drain(0..self.log.len() - 200);
        }
    }
}

// ── Messages ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Msg {
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

// ── Update ────────────────────────────────────────────────────────────────────

pub fn update(app: &mut App, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::NavPanel(p) => app.active_panel = p,

        Msg::RefreshModules => {
            app.modules = discover_modules();
            if app
                .selected_module
                .as_ref()
                .map(|s| !app.modules.contains(s))
                .unwrap_or(true)
            {
                app.selected_module = app.modules.first().cloned();
            }
            app.push_log(
                format!("{} module(s) found.", app.modules.len()),
                LogLevel::Info,
            );
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

        Msg::BrowseDir => {
            return Task::perform(
                async { rfd::AsyncFileDialog::new().pick_folder().await },
                |h| Msg::DirPicked(h.map(|x| x.path().to_path_buf())),
            );
        }

        Msg::DirPicked(p) => {
            if let Some(p) = p {
                app.directory = p.display().to_string();
            }
        }

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

        Msg::OpSelected(o) => {
            app.op = o;
            app.op_output.clear();
            app.op_confirm.clear();
        }

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
                Ok(n) => app.push_log(format!("{n} file(s) affected."), LogLevel::Ok),
                Err(e) => app.push_log(format!("Error: {e}"), LogLevel::Err),
            }
        }
    }

    Task::none()
}
