// ─────────────────────────────────────────────────────────────────────────────
// Module — JSON classification file types and discovery
// ─────────────────────────────────────────────────────────────────────────────

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::domain::{ModLoader, Side};

// ── Public types ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleEntry {
    pub mod_version: String,
    pub mod_tag: Side,
    pub mod_type: ModLoader,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub version: f64,
    pub author: String,
    pub mods: BTreeMap<String, ModuleEntry>,
    pub path: String,
}

impl Module {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let raw: ModuleJson = serde_json::from_str(&fs::read_to_string(path)?)?;
        Ok(Self {
            name: raw.header.module_name,
            version: raw.header.module_version,
            author: raw.header.module_author,
            mods: raw.mods,
            path: path.to_string(),
        })
    }
}

// ── Private JSON shape ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
struct ModuleHeader {
    module_name: String,
    module_version: f64,
    module_author: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ModuleJson {
    header: ModuleHeader,
    mods: BTreeMap<String, ModuleEntry>,
}

// ── Discovery ─────────────────────────────────────────────────────────────────

/// Search for module JSON files next to the executable and in `./modules/`.
pub fn discover_modules() -> Vec<String> {
    let exe_base = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("."));

    let mut found = Vec::new();

    // Check for a standalone test.json first
    for tp in &[exe_base.join("test.json"), PathBuf::from("test.json")] {
        if tp.exists() {
            found.push(tp.display().to_string());
            break;
        }
    }

    // Then scan the modules/ directory
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
