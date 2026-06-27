// ─────────────────────────────────────────────────────────────────────────────
// Core domain enums shared across the whole crate
// ─────────────────────────────────────────────────────────────────────────────

use serde::{Deserialize, Serialize};

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
            ModLoader::Unknown => "Unknown",
            ModLoader::Forge => "Forge",
            ModLoader::NeoForge => "NeoForge",
            ModLoader::Fabric => "Fabric",
            ModLoader::Quilt => "Quilt",
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
            Side::Client => "Client",
            Side::Server => "Server",
            Side::Both => "Both",
        })
    }
}
