// ─────────────────────────────────────────────────────────────────────────────
// Scanner — jar parsing and directory scanning
// ─────────────────────────────────────────────────────────────────────────────

use std::fs;
use std::io::Read;

use iced::Color;

use crate::bytecode;
use crate::domain::{ModLoader, Side};
use crate::module::{Module, ModuleEntry};
use crate::palette as pal;

// ── Jar metadata ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct JarInfo {
    pub mod_id: String,
    pub loader: ModLoader,
    pub version: Option<String>,
    pub declared_side: Option<Side>,
}

// ── Match quality ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchQuality {
    Full,
    Partial,
    Unidentified,
    Unknown,
}

// ── Scan result ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ScanResult {
    pub jar_name: String,
    pub jar_info: Option<JarInfo>,
    pub parse_error: Option<String>,
    pub module_entry: Option<ModuleEntry>,
    pub match_quality: MatchQuality,
    /// Side inferred purely from bytecode analysis (no module required)
    pub bytecode_side: Option<bytecode::DetectedSide>,
    pub bytecode_confidence: bytecode::Confidence,
    /// A representative signal string shown in the UI
    pub bytecode_signal: Option<String>,
}

impl ScanResult {
    pub fn status_label(&self) -> &'static str {
        match self.match_quality {
            MatchQuality::Full => "Full match",
            MatchQuality::Partial => "Partial",
            MatchQuality::Unidentified => "Unidentified",
            MatchQuality::Unknown => "Unknown",
        }
    }

    pub fn status_color(&self) -> Color {
        match self.match_quality {
            MatchQuality::Full => pal::GREEN,
            MatchQuality::Partial => pal::AMBER,
            MatchQuality::Unidentified => pal::RED,
            MatchQuality::Unknown => pal::FAINT,
        }
    }

    /// Side in priority order: module entry → manifest declaration → bytecode.
    pub fn effective_side(&self) -> Side {
        if let Some(s) = self.module_entry.as_ref().map(|e| e.mod_tag) {
            return s;
        }
        if let Some(s) = self.jar_info.as_ref().and_then(|i| i.declared_side) {
            return s;
        }
        match &self.bytecode_side {
            Some(bytecode::DetectedSide::Client) => Side::Client,
            Some(bytecode::DetectedSide::Server) => Side::Server,
            Some(bytecode::DetectedSide::Both) => Side::Both,
            _ => Side::Unknown,
        }
    }

    /// Human-readable source of the side determination.
    pub fn side_source(&self) -> &'static str {
        if self.module_entry.is_some() {
            return "module";
        }
        if self
            .jar_info
            .as_ref()
            .and_then(|i| i.declared_side)
            .is_some()
        {
            return "manifest";
        }
        match self.bytecode_confidence {
            bytecode::Confidence::Annotation => "annotation",
            bytecode::Confidence::ClassReference => "bytecode",
            bytecode::Confidence::None => "—",
        }
    }
}

// ── Scan summary ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
pub struct ScanSummary {
    pub total: usize,
    pub full: usize,
    pub partial: usize,
    pub unidentified: usize,
    pub unknown: usize,
}

// ── Directory scan ────────────────────────────────────────────────────────────

pub fn scan_directory(dir: &str, module: &Module) -> (Vec<ScanResult>, ScanSummary) {
    let mut jars: Vec<String> = fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("jar"))
                .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                .collect()
        })
        .unwrap_or_default();
    jars.sort();

    let mut results = Vec::new();
    for jar_name in jars {
        let path = format!("{}/{}", dir.trim_end_matches('/'), jar_name);

        let (jar_info, parse_error) = match parse_jar(&path) {
            Ok(i) => (i, None),
            Err(e) => (None, Some(e.to_string())),
        };

        // Bytecode analysis runs regardless of whether a module is loaded
        let bc = bytecode::analyse_jar(&path).unwrap_or_else(bytecode::BytecodeEvidence::unknown);

        let (module_entry, match_quality) = if let Some(info) = &jar_info {
            if let Some(entry) = module.mods.get(&info.mod_id).cloned() {
                let version_ok = entry.mod_version == "*"
                    || info
                        .version
                        .as_deref()
                        .map(|v| v == entry.mod_version)
                        .unwrap_or(false);
                let loader_ok = info.loader == entry.mod_type;
                let q = if version_ok && loader_ok {
                    MatchQuality::Full
                } else {
                    MatchQuality::Partial
                };
                (Some(entry), q)
            } else {
                (None, MatchQuality::Unidentified)
            }
        } else {
            (None, MatchQuality::Unknown)
        };

        results.push(ScanResult {
            jar_name,
            jar_info,
            parse_error,
            module_entry,
            match_quality,
            bytecode_side: Some(bc.side),
            bytecode_confidence: bc.confidence,
            bytecode_signal: bc.signal,
        });
    }

    let summary = ScanSummary {
        total: results.len(),
        full: results
            .iter()
            .filter(|r| r.match_quality == MatchQuality::Full)
            .count(),
        partial: results
            .iter()
            .filter(|r| r.match_quality == MatchQuality::Partial)
            .count(),
        unidentified: results
            .iter()
            .filter(|r| r.match_quality == MatchQuality::Unidentified)
            .count(),
        unknown: results
            .iter()
            .filter(|r| r.match_quality == MatchQuality::Unknown)
            .count(),
    };

    (results, summary)
}

// ── Private parse helpers ─────────────────────────────────────────────────────

fn read_zip_entry(e: &mut zip::read::ZipFile) -> Result<String, Box<dyn std::error::Error>> {
    let mut s = String::new();
    e.read_to_string(&mut s)?;
    Ok(s)
}

fn toml_str(v: &toml::Value) -> Option<String> {
    v.as_str()
        .map(String::from)
        .or_else(|| v.as_float().map(|f| f.to_string()))
        .or_else(|| v.as_integer().map(|i| i.to_string()))
}

fn json_str(v: &serde_json::Value) -> Option<String> {
    v.as_str()
        .map(String::from)
        .or_else(|| v.as_f64().map(|f| f.to_string()))
}

fn parse_jar(path: &str) -> Result<Option<JarInfo>, Box<dyn std::error::Error>> {
    let mut archive = zip::ZipArchive::new(fs::File::open(path)?)?;
    for i in 0..archive.len() {
        let mut e = archive.by_index(i)?;
        let name = e.name().to_string();
        if name.ends_with("mods.toml") {
            return parse_forge(&mut e).map(Some);
        }
        if name.ends_with("fabric.mod.json") {
            return parse_fabric(&mut e).map(Some);
        }
        if name.ends_with("quilt.mod.json") {
            return parse_quilt(&mut e).map(Some);
        }
        if name.ends_with("mcmod.info") {
            return parse_legacy(&mut e).map(Some);
        }
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
    let first = parsed
        .get("mods")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first());
    let mod_id = first
        .and_then(|m| m.get("modId"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .unwrap_or_else(|| "unknown".into());
    let version = first
        .and_then(|m| m.get("version").or_else(|| m.get("modVersion")))
        .and_then(toml_str);
    Ok(JarInfo {
        mod_id,
        loader,
        version,
        declared_side: None,
    })
}

fn parse_fabric(e: &mut zip::read::ZipFile) -> Result<JarInfo, Box<dyn std::error::Error>> {
    let v: serde_json::Value = serde_json::from_str(&read_zip_entry(e)?)?;
    let mod_id = v
        .get("id")
        .and_then(|x| x.as_str())
        .map(String::from)
        .unwrap_or_else(|| "unknown".into());
    let version = v.get("version").and_then(json_str);
    let declared_side = v
        .get("environment")
        .and_then(|x| x.as_str())
        .and_then(|s| match s {
            "client" => Some(Side::Client),
            "server" => Some(Side::Server),
            "*" => Some(Side::Both),
            _ => None,
        });
    Ok(JarInfo {
        mod_id,
        loader: ModLoader::Fabric,
        version,
        declared_side,
    })
}

fn parse_quilt(e: &mut zip::read::ZipFile) -> Result<JarInfo, Box<dyn std::error::Error>> {
    let v: serde_json::Value = serde_json::from_str(&read_zip_entry(e)?)?;
    let ql = v.get("quilt_loader");
    let mod_id = ql
        .and_then(|l| l.get("id"))
        .and_then(|x| x.as_str())
        .map(String::from)
        .unwrap_or_else(|| "unknown".into());
    let version = ql.and_then(|l| l.get("version")).and_then(json_str);
    Ok(JarInfo {
        mod_id,
        loader: ModLoader::Quilt,
        version,
        declared_side: None,
    })
}

fn parse_legacy(e: &mut zip::read::ZipFile) -> Result<JarInfo, Box<dyn std::error::Error>> {
    let v: serde_json::Value = serde_json::from_str(&read_zip_entry(e)?)?;
    let first = v.as_array().and_then(|a| a.first());
    let mod_id = first
        .and_then(|m| m.get("modid"))
        .and_then(|x| x.as_str())
        .map(String::from)
        .unwrap_or_else(|| "unknown".into());
    let version = first.and_then(|m| m.get("version")).and_then(json_str);
    Ok(JarInfo {
        mod_id,
        loader: ModLoader::Forge,
        version,
        declared_side: None,
    })
}
