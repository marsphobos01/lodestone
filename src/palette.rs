// ─────────────────────────────────────────────────────────────────────────────
// Palette — dark pro theme
//
// Base:   very dark blue-black  #0c0c10
// Cards:  lifted dark surface   #1c1c28
// Accent: electric orange       #f97316
// Status: vibrant semantic hues
// ─────────────────────────────────────────────────────────────────────────────

use iced::Color;

// ── Backgrounds ───────────────────────────────────────────────────────────────
pub const BG: Color = Color {
    r: 0.047,
    g: 0.047,
    b: 0.063,
    a: 1.0,
}; // #0c0c10
pub const BG_WARM: Color = Color {
    r: 0.078,
    g: 0.078,
    b: 0.125,
    a: 1.0,
}; // #141420 — header/footer
pub const SURFACE: Color = Color {
    r: 0.110,
    g: 0.110,
    b: 0.157,
    a: 1.0,
}; // #1c1c28 — cards

// ── Borders ───────────────────────────────────────────────────────────────────
pub const LINE: Color = Color {
    r: 0.165,
    g: 0.165,
    b: 0.243,
    a: 1.0,
}; // #2a2a3e
pub const LINE_DIM: Color = Color {
    r: 0.118,
    g: 0.118,
    b: 0.173,
    a: 1.0,
}; // #1e1e2c

// ── Text ──────────────────────────────────────────────────────────────────────
pub const INK: Color = Color {
    r: 0.910,
    g: 0.894,
    b: 0.957,
    a: 1.0,
}; // #e8e4f4
pub const MUTED: Color = Color {
    r: 0.580,
    g: 0.573,
    b: 0.706,
    a: 1.0,
}; // #9492b4
pub const FAINT: Color = Color {
    r: 0.337,
    g: 0.329,
    b: 0.439,
    a: 1.0,
}; // #565470

// ── Accent — electric orange ──────────────────────────────────────────────────
pub const ACCENT: Color = Color {
    r: 0.976,
    g: 0.451,
    b: 0.086,
    a: 1.0,
}; // #f97316
pub const ACCENT_DARK: Color = Color {
    r: 0.769,
    g: 0.361,
    b: 0.094,
    a: 1.0,
}; // #c45c18
pub const ACCENT_TINT: Color = Color {
    r: 0.976,
    g: 0.451,
    b: 0.086,
    a: 0.12,
};

// ── Status ────────────────────────────────────────────────────────────────────
pub const GREEN: Color = Color {
    r: 0.204,
    g: 0.827,
    b: 0.600,
    a: 1.0,
}; // #34d399
pub const AMBER: Color = Color {
    r: 0.984,
    g: 0.749,
    b: 0.141,
    a: 1.0,
}; // #fbbf24
pub const RED: Color = Color {
    r: 0.973,
    g: 0.443,
    b: 0.443,
    a: 1.0,
}; // #f87171
pub const PURPLE: Color = Color {
    r: 0.655,
    g: 0.545,
    b: 0.980,
    a: 1.0,
}; // #a78bfa
