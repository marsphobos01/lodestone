// ─────────────────────────────────────────────────────────────────────────────
// ui.rs — dark pro aesthetic, complete visual overhaul
// ─────────────────────────────────────────────────────────────────────────────

use iced::alignment;
use iced::theme::Theme;
use iced::widget::{
    button, column, container, horizontal_rule, pick_list, row, scrollable, text, text_input, Space,
};
use iced::{Color, Element, Length};

use crate::app::{App, LogLevel, Msg, Panel};
use crate::domain::{ModLoader, Side};
use crate::operations::Operation;
use crate::palette as pal;
use crate::scanner::ScanResult;

// ─────────────────────────────────────────────────────────────────────────────
// Micro utilities
// ─────────────────────────────────────────────────────────────────────────────

fn tc(c: Color) -> impl Fn(&Theme) -> iced::widget::text::Style {
    move |_| iced::widget::text::Style { color: Some(c) }
}

/// Same hue as `base` at `alpha` — used for badge tints and glows.
fn tint(base: Color, alpha: f32) -> Color {
    Color { a: alpha, ..base }
}

// ─────────────────────────────────────────────────────────────────────────────
// Glowing pill badge — the centrepiece visual element
// ─────────────────────────────────────────────────────────────────────────────

fn glow_badge<'a>(label: &'static str, fg: Color) -> Element<'a, Msg> {
    container(text(label).size(11).style(tc(fg)))
        .style(move |_| container::Style {
            background: Some(tint(fg, 0.10).into()),
            border: iced::border::Border {
                color: tint(fg, 0.38),
                width: 1.0,
                radius: 999.0.into(),
            },
            shadow: iced::Shadow {
                color: tint(fg, 0.22),
                offset: iced::Vector::new(0.0, 0.0),
                blur_radius: 10.0,
            },
            ..Default::default()
        })
        .padding([3, 10])
        .into()
}

fn side_badge<'a>(side: Side) -> Element<'a, Msg> {
    match side {
        Side::Client => glow_badge("Client", pal::ACCENT),
        Side::Server => glow_badge("Server", pal::GREEN),
        Side::Both => glow_badge("Both", pal::PURPLE),
        Side::Unknown => glow_badge("Unknown", pal::FAINT),
    }
}

fn loader_label<'a>(loader: ModLoader) -> Element<'a, Msg> {
    let (label, fg) = match loader {
        ModLoader::Fabric => ("Fabric", pal::ACCENT),
        ModLoader::Quilt => ("Quilt", pal::PURPLE),
        ModLoader::Forge => ("Forge", pal::AMBER),
        ModLoader::NeoForge => ("NeoForge", pal::AMBER),
        ModLoader::Unknown => ("—", pal::FAINT),
    };
    text(label).size(12).style(tc(fg)).into()
}

fn match_indicator<'a>(label: &'static str, color: Color) -> Element<'a, Msg> {
    row![
        text("●").size(8).style(tc(color)),
        Space::with_width(5),
        text(label).size(12).style(tc(color)),
    ]
    .align_y(alignment::Vertical::Center)
    .into()
}

fn source_chip<'a>(source: &'a str) -> Element<'a, Msg> {
    let fg = match source {
        "module" => pal::INK,
        "manifest" => pal::ACCENT,
        "annotation" => pal::GREEN,
        "bytecode" => pal::PURPLE,
        _ => pal::FAINT,
    };
    text(source).size(11).style(tc(fg)).into()
}

// ─────────────────────────────────────────────────────────────────────────────
// Card container (lifted dark surface)
// ─────────────────────────────────────────────────────────────────────────────

fn card(content: Element<'_, Msg>) -> Element<'_, Msg> {
    container(content)
        .style(|_| container::Style {
            background: Some(pal::SURFACE.into()),
            border: iced::border::Border {
                color: pal::LINE,
                width: 1.0,
                radius: 12.0.into(),
            },
            ..Default::default()
        })
        .padding(18)
        .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// Buttons
// ─────────────────────────────────────────────────────────────────────────────

fn btn_primary<'a>(label: &'a str) -> button::Button<'a, Msg> {
    button(text(label).size(13).style(tc(Color::WHITE)))
        .style(|_, status| {
            let a = match status {
                button::Status::Hovered => 0.82,
                button::Status::Pressed => 0.65,
                button::Status::Disabled => 0.30,
                button::Status::Active => 1.0,
            };
            button::Style {
                background: Some(tint(pal::ACCENT, a).into()),
                text_color: Color::WHITE,
                border: iced::border::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 8.0.into(),
                },
                shadow: iced::Shadow {
                    color: tint(pal::ACCENT, 0.35 * a),
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 8.0,
                },
            }
        })
        .padding([9, 18])
}

fn btn_danger<'a>(label: &'a str) -> button::Button<'a, Msg> {
    button(text(label).size(13).style(tc(Color::WHITE)))
        .style(|_, status| {
            let a = match status {
                button::Status::Hovered => 0.80,
                button::Status::Pressed => 0.64,
                button::Status::Disabled => 0.30,
                button::Status::Active => 1.0,
            };
            button::Style {
                background: Some(tint(pal::RED, a).into()),
                text_color: Color::WHITE,
                border: iced::border::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: 8.0.into(),
                },
                shadow: iced::Shadow {
                    color: tint(pal::RED, 0.35 * a),
                    offset: iced::Vector::new(0.0, 2.0),
                    blur_radius: 8.0,
                },
            }
        })
        .padding([9, 18])
}

fn btn_ghost<'a>(label: &'a str) -> button::Button<'a, Msg> {
    button(text(label).size(13).style(tc(pal::MUTED)))
        .style(|_, status| {
            let bg = match status {
                button::Status::Hovered => tint(pal::LINE, 1.0),
                button::Status::Pressed => pal::LINE,
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
        .padding([9, 14])
}

/// Navigation tab: active = solid accent fill with glow.
fn btn_nav<'a>(label: &'a str, active: bool, msg: Msg) -> Element<'a, Msg> {
    let (bg, fg) = if active {
        (pal::ACCENT, Color::WHITE)
    } else {
        (Color::TRANSPARENT, pal::MUTED)
    };
    let shadow = if active {
        iced::Shadow {
            color: tint(pal::ACCENT, 0.30),
            offset: iced::Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        }
    } else {
        iced::Shadow::default()
    };
    button(text(label).size(13).style(tc(fg)))
        .style(move |_, _| button::Style {
            background: Some(bg.into()),
            text_color: fg,
            border: iced::border::Border {
                color: if active {
                    Color::TRANSPARENT
                } else {
                    pal::LINE
                },
                width: if active { 0.0 } else { 1.0 },
                radius: 8.0.into(),
            },
            shadow,
        })
        .on_press(msg)
        .padding([7, 18])
        .into()
}

/// Filter chip (pill shaped).
fn filter_chip<'a>(label: String, active: bool, msg: Msg) -> Element<'a, Msg> {
    let (bg, fg, bdr) = if active {
        (
            tint(pal::ACCENT, 0.12),
            pal::ACCENT,
            tint(pal::ACCENT, 0.45),
        )
    } else {
        (Color::TRANSPARENT, pal::MUTED, pal::LINE_DIM)
    };
    button(text(label).size(12).style(tc(fg)))
        .style(move |_, _| button::Style {
            background: Some(bg.into()),
            text_color: fg,
            border: iced::border::Border {
                color: bdr,
                width: 1.0,
                radius: 999.0.into(),
            },
            ..Default::default()
        })
        .on_press(msg)
        .padding([5, 14])
        .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// Form controls
// ─────────────────────────────────────────────────────────────────────────────

fn field_label<'a>(label: &'a str) -> Element<'a, Msg> {
    text(label).size(11).style(tc(pal::MUTED)).into()
}

fn input_style(status: iced::widget::text_input::Status) -> iced::widget::text_input::Style {
    let (border_color, border_width) = match status {
        iced::widget::text_input::Status::Focused => (pal::ACCENT, 1.5),
        iced::widget::text_input::Status::Hovered => (pal::LINE, 1.0),
        _ => (pal::LINE_DIM, 1.0),
    };
    iced::widget::text_input::Style {
        background: pal::BG.into(),
        border: iced::border::Border {
            color: border_color,
            width: border_width,
            radius: 8.0.into(),
        },
        icon: pal::FAINT,
        placeholder: pal::FAINT,
        value: pal::INK,
        selection: tint(pal::ACCENT, 0.22),
    }
}

fn input_style_danger(status: iced::widget::text_input::Status) -> iced::widget::text_input::Style {
    let border_width = match status {
        iced::widget::text_input::Status::Focused => 1.5,
        _ => 1.0,
    };
    iced::widget::text_input::Style {
        border: iced::border::Border {
            color: pal::RED,
            width: border_width,
            radius: 8.0.into(),
        },
        ..input_style(status)
    }
}

fn pick_style() -> iced::widget::pick_list::Style {
    iced::widget::pick_list::Style {
        text_color: pal::INK,
        placeholder_color: pal::FAINT,
        handle_color: pal::MUTED,
        background: pal::BG.into(),
        border: iced::border::Border {
            color: pal::LINE,
            width: 1.0,
            radius: 8.0.into(),
        },
    }
}

fn thin_rule<'a>() -> Element<'a, Msg> {
    container(horizontal_rule(1))
        .style(|_| container::Style {
            ..Default::default()
        })
        .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// Section header with accent left-bar
// ─────────────────────────────────────────────────────────────────────────────

fn section_header<'a>(label: &'a str) -> Element<'a, Msg> {
    row![
        // Accent left stripe
        container(Space::with_height(12))
            .width(3)
            .style(|_| container::Style {
                background: Some(pal::ACCENT.into()),
                border: iced::border::Border {
                    radius: 999.0.into(),
                    ..Default::default()
                },
                text_color: None,
                shadow: iced::Shadow::default(),
            }),
        Space::with_width(8),
        text(label).size(11).style(tc(pal::MUTED)),
    ]
    .align_y(alignment::Vertical::Center)
    .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// Scan stat block (shown in topbar after scan)
// ─────────────────────────────────────────────────────────────────────────────

fn stat_block<'a>(value: usize, label: &'a str, value_color: Color) -> Element<'a, Msg> {
    column![
        text(value.to_string()).size(20).style(tc(value_color)),
        text(label).size(10).style(tc(pal::FAINT)),
    ]
    .align_x(alignment::Horizontal::Center)
    .spacing(1)
    .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// View — top bar
// ─────────────────────────────────────────────────────────────────────────────

fn view_topbar(app: &App) -> Element<'_, Msg> {
    // Brand mark
    let brand = row![
        text("◈").size(18).style(tc(pal::ACCENT)),
        Space::with_width(8),
        text("LODESTONE").size(15).style(tc(pal::INK)),
        Space::with_width(6),
        text("v2").size(11).style(tc(pal::FAINT)),
    ]
    .align_y(alignment::Vertical::Center);

    // Navigation
    let nav = row![
        btn_nav(
            "Scan",
            app.active_panel == Panel::Scan,
            Msg::NavPanel(Panel::Scan)
        ),
        btn_nav(
            "Operate",
            app.active_panel == Panel::Operate,
            Msg::NavPanel(Panel::Operate)
        ),
    ]
    .spacing(6);

    // Right side — stats when scan has run, otherwise placeholder
    let stats: Element<'_, Msg> = if app.scan_results.is_empty() {
        text("no scan").size(11).style(tc(pal::FAINT)).into()
    } else {
        let matched = app.summary.full + app.summary.partial;
        let unid_color = if app.summary.unidentified > 0 {
            pal::AMBER
        } else {
            pal::FAINT
        };
        container(
            row![
                stat_block(app.summary.total, "jars", pal::INK),
                Space::with_width(2),
                text("·").size(16).style(tc(pal::LINE)),
                Space::with_width(2),
                stat_block(matched, "matched", pal::GREEN),
                Space::with_width(2),
                text("·").size(16).style(tc(pal::LINE)),
                Space::with_width(2),
                stat_block(app.summary.unidentified, "unknown", unid_color),
            ]
            .spacing(10)
            .align_y(alignment::Vertical::Center),
        )
        .style(|_| container::Style {
            background: Some(tint(pal::LINE_DIM, 1.0).into()),
            border: iced::border::Border {
                color: pal::LINE,
                width: 1.0,
                radius: 10.0.into(),
            },
            ..Default::default()
        })
        .padding([8, 16])
        .into()
    };

    container(
        row![
            brand,
            Space::with_width(32),
            nav,
            Space::with_width(Length::Fill),
            stats,
        ]
        .align_y(alignment::Vertical::Center),
    )
    .padding([12, 28])
    .style(|_| container::Style {
        background: Some(pal::BG_WARM.into()),
        ..Default::default()
    })
    .width(Length::Fill)
    .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// View — status bar (bottom)
// ─────────────────────────────────────────────────────────────────────────────

fn view_statusbar(app: &App) -> Element<'_, Msg> {
    let (msg_text, dot_color) = match app.log.last() {
        None => ("Ready.".to_string(), pal::FAINT),
        Some((t, lv)) => {
            let c = match lv {
                LogLevel::Ok => pal::GREEN,
                LogLevel::Warn => pal::AMBER,
                LogLevel::Err => pal::RED,
                LogLevel::Info => pal::FAINT,
            };
            (t.clone(), c)
        }
    };

    container(
        row![
            text("◆").size(8).style(tc(dot_color)),
            Space::with_width(8),
            text(msg_text).size(11).style(tc(pal::MUTED)),
        ]
        .align_y(alignment::Vertical::Center),
    )
    .padding([7, 28])
    .style(|_| container::Style {
        background: Some(pal::BG_WARM.into()),
        ..Default::default()
    })
    .width(Length::Fill)
    .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// View — sidebar (module + directory controls)
// ─────────────────────────────────────────────────────────────────────────────

fn view_sidebar(app: &App) -> Element<'_, Msg> {
    // Module loaded indicator
    let module_info: Element<'_, Msg> = match &app.loaded_module {
        Some(m) => column![
            row![
                text("●").size(8).style(tc(pal::GREEN)),
                Space::with_width(7),
                text(&m.name).size(13).style(tc(pal::INK)),
            ]
            .align_y(alignment::Vertical::Center),
            Space::with_height(3),
            text(format!(
                "v{}  ·  {}  ·  {} entries",
                m.version,
                m.author,
                m.mods.len()
            ))
            .size(11)
            .style(tc(pal::FAINT)),
        ]
        .spacing(0)
        .into(),
        None => row![
            text("●").size(8).style(tc(pal::FAINT)),
            Space::with_width(7),
            text("No module loaded").size(12).style(tc(pal::FAINT)),
        ]
        .align_y(alignment::Vertical::Center)
        .into(),
    };

    let module_card = card(
        column![
            section_header("MODULE"),
            Space::with_height(12),
            pick_list(
                app.modules.clone(),
                app.selected_module.clone(),
                Msg::ModuleSelected,
            )
            .placeholder("Select a module file…")
            .style(|_, _| pick_style())
            .width(Length::Fill),
            Space::with_height(8),
            row![
                btn_ghost("Refresh").on_press(Msg::RefreshModules),
                Space::with_width(Length::Fill),
                btn_primary("Load").on_press(Msg::LoadModule),
            ]
            .align_y(alignment::Vertical::Center),
            Space::with_height(14),
            thin_rule(),
            Space::with_height(12),
            module_info,
        ]
        .spacing(0)
        .into(),
    );

    let dir_card = card(
        column![
            section_header("DIRECTORY"),
            Space::with_height(12),
            row![
                text_input("Path to mods folder…", &app.directory)
                    .on_input(Msg::DirChanged)
                    .style(|_, s| input_style(s))
                    .padding([9, 12])
                    .size(12),
                Space::with_width(8),
                btn_ghost("…").on_press(Msg::BrowseDir),
            ]
            .align_y(alignment::Vertical::Center),
            Space::with_height(8),
            btn_primary("Scan directory")
                .on_press(Msg::ScanDir)
                .width(Length::Fill),
        ]
        .spacing(0)
        .into(),
    );

    column![module_card, Space::with_height(12), dir_card]
        .spacing(0)
        .width(270)
        .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// View — results (filter chips + table)
// ─────────────────────────────────────────────────────────────────────────────

fn view_results(app: &App) -> Element<'_, Msg> {
    let n = app.scan_results.len();
    let count = |pred: fn(&ScanResult) -> bool| -> usize {
        app.scan_results.iter().filter(|r| pred(r)).count()
    };
    let nc = count(|r| r.effective_side() == Side::Client);
    let ns = count(|r| r.effective_side() == Side::Server);
    let nb = count(|r| r.effective_side() == Side::Both);
    let nu = count(|r| r.effective_side() == Side::Unknown);

    let chip = |label: &str, cnt: usize, active: bool, msg: Msg| -> Element<'_, Msg> {
        let s = if n == 0 {
            label.to_string()
        } else {
            format!("{label}  {cnt}")
        };
        filter_chip(s, active, msg)
    };

    let chips = row![
        chip("All", n, app.filter_side.is_none(), Msg::FilterSide(None)),
        chip(
            "Client",
            nc,
            app.filter_side == Some(Side::Client),
            Msg::FilterSide(Some(Side::Client))
        ),
        chip(
            "Server",
            ns,
            app.filter_side == Some(Side::Server),
            Msg::FilterSide(Some(Side::Server))
        ),
        chip(
            "Both",
            nb,
            app.filter_side == Some(Side::Both),
            Msg::FilterSide(Some(Side::Both))
        ),
        chip(
            "Unknown",
            nu,
            app.filter_side == Some(Side::Unknown),
            Msg::FilterSide(Some(Side::Unknown))
        ),
    ]
    .spacing(6);

    let filtered: Vec<&ScanResult> = app
        .scan_results
        .iter()
        .filter(|r| {
            app.filter_side
                .map(|s| r.effective_side() == s)
                .unwrap_or(true)
        })
        .collect();

    let table: Element<'_, Msg> = if app.scan_results.is_empty() {
        // Empty state
        container(
            column![
                text("◈").size(40).style(tc(pal::LINE)),
                Space::with_height(16),
                text("Nothing scanned yet").size(16).style(tc(pal::MUTED)),
                Space::with_height(6),
                text("Load a module, pick a mods directory, then hit Scan.")
                    .size(12)
                    .style(tc(pal::FAINT)),
            ]
            .spacing(0)
            .align_x(alignment::Horizontal::Center),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .height(300)
        .into()
    } else {
        // ── Table header ─────────────────────────────────────────────────────
        let hdr_col = |label: &str, portion: u16| -> Element<'_, Msg> {
            text(label.to_string())
                .size(10)
                .style(tc(pal::FAINT))
                .width(Length::FillPortion(portion))
                .into()
        };

        let header = container(
            row![
                hdr_col("FILE", 5),
                hdr_col("MOD ID", 3),
                hdr_col("LOADER", 2),
                hdr_col("SIDE", 2),
                hdr_col("SOURCE", 2),
                hdr_col("MATCH", 2),
            ]
            .spacing(12),
        )
        .padding([10, 16])
        .style(|_| container::Style {
            background: Some(pal::BG_WARM.into()),
            border: iced::border::Border {
                color: pal::LINE,
                width: 1.0,
                radius: iced::border::Radius {
                    top_left: 10.0,
                    top_right: 10.0,
                    bottom_left: 0.0,
                    bottom_right: 0.0,
                },
            },
            ..Default::default()
        });

        // ── Rows ─────────────────────────────────────────────────────────────
        let total = filtered.len();
        let rows: Vec<Element<'_, Msg>> = filtered
            .iter()
            .enumerate()
            .map(|(i, r)| {
                // Alternating row backgrounds
                let bg = if i % 2 == 0 {
                    pal::BG
                } else {
                    tint(pal::SURFACE, 0.55)
                };
                let is_last = i == total - 1;
                let radius: iced::border::Radius = if is_last {
                    iced::border::Radius {
                        top_left: 0.0,
                        top_right: 0.0,
                        bottom_left: 10.0,
                        bottom_right: 10.0,
                    }
                } else {
                    0.0.into()
                };

                let mod_id = r
                    .jar_info
                    .as_ref()
                    .map(|j| j.mod_id.as_str())
                    .unwrap_or("—");
                let loader = r
                    .jar_info
                    .as_ref()
                    .map(|j| j.loader)
                    .unwrap_or(ModLoader::Unknown);
                let side = r.effective_side();
                let source = r.side_source();

                container(
                    row![
                        text(&r.jar_name)
                            .size(12)
                            .style(tc(pal::INK))
                            .width(Length::FillPortion(5)),
                        text(mod_id)
                            .size(12)
                            .style(tc(pal::MUTED))
                            .width(Length::FillPortion(3)),
                        container(loader_label(loader))
                            .width(Length::FillPortion(2))
                            .align_y(alignment::Vertical::Center),
                        container(side_badge(side))
                            .width(Length::FillPortion(2))
                            .align_y(alignment::Vertical::Center),
                        container(source_chip(source))
                            .width(Length::FillPortion(2))
                            .align_y(alignment::Vertical::Center),
                        container(match_indicator(r.status_label(), r.status_color()))
                            .width(Length::FillPortion(2))
                            .align_y(alignment::Vertical::Center),
                    ]
                    .spacing(12)
                    .align_y(alignment::Vertical::Center),
                )
                .padding([11, 16])
                .style(move |_| container::Style {
                    background: Some(bg.into()),
                    border: iced::border::Border {
                        color: pal::LINE_DIM,
                        width: 1.0,
                        radius,
                    },
                    ..Default::default()
                })
                .into()
            })
            .collect();

        column![header, column(rows).spacing(0)].spacing(0).into()
    };

    column![chips, Space::with_height(14), table]
        .spacing(0)
        .width(Length::Fill)
        .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// View — Scan panel
// ─────────────────────────────────────────────────────────────────────────────

fn view_scan(app: &App) -> Element<'_, Msg> {
    row![view_sidebar(app), Space::with_width(24), view_results(app),].into()
}

// ─────────────────────────────────────────────────────────────────────────────
// View — Operate panel
// ─────────────────────────────────────────────────────────────────────────────

fn view_operate(app: &App) -> Element<'_, Msg> {
    let affected = app
        .scan_results
        .iter()
        .filter(|r| r.effective_side() == app.op_side)
        .count();

    // ── Target + operation pickers ────────────────────────────────────────
    let pickers = card(
        column![
            section_header("ACTION"),
            Space::with_height(14),
            row![
                column![
                    field_label("Target side"),
                    Space::with_height(6),
                    pick_list(
                        vec![Side::Client, Side::Server, Side::Both, Side::Unknown],
                        Some(app.op_side),
                        Msg::OpSideSelected,
                    )
                    .style(|_, _| pick_style())
                    .width(Length::Fill),
                ]
                .spacing(0)
                .width(Length::FillPortion(1)),
                Space::with_width(14),
                column![
                    field_label("Operation"),
                    Space::with_height(6),
                    pick_list(
                        vec![
                            Operation::Zip,
                            Operation::Move,
                            Operation::Delete,
                            Operation::Export,
                        ],
                        Some(app.op),
                        Msg::OpSelected,
                    )
                    .style(|_, _| pick_style())
                    .width(Length::Fill),
                ]
                .spacing(0)
                .width(Length::FillPortion(1)),
            ],
        ]
        .spacing(0)
        .into(),
    );

    // ── Output / confirmation ──────────────────────────────────────────────
    let output_card: Element<'_, Msg> = if app.op == Operation::Delete {
        card(
            column![
                section_header("CONFIRMATION"),
                Space::with_height(10),
                // Warning box
                container(
                    row![
                        text("⚠").size(13).style(tc(pal::AMBER)),
                        Space::with_width(10),
                        text("This permanently deletes files. Type DELETE to confirm.")
                            .size(12)
                            .style(tc(pal::AMBER)),
                    ]
                    .align_y(alignment::Vertical::Center),
                )
                .style(|_| container::Style {
                    background: Some(tint(pal::AMBER, 0.08).into()),
                    border: iced::border::Border {
                        color: tint(pal::AMBER, 0.25),
                        width: 1.0,
                        radius: 8.0.into(),
                    },
                    ..Default::default()
                })
                .padding([10, 14])
                .width(Length::Fill),
                Space::with_height(10),
                text_input("Type DELETE…", &app.op_confirm)
                    .on_input(Msg::OpConfirmChanged)
                    .style(|_, s| input_style_danger(s))
                    .padding([10, 12])
                    .size(13),
            ]
            .spacing(0)
            .into(),
        )
    } else {
        let placeholder = match app.op {
            Operation::Zip => "Output .zip path…",
            Operation::Move => "Destination folder…",
            Operation::Export => "Output .txt path…",
            Operation::Delete => unreachable!(),
        };
        card(
            column![
                section_header("OUTPUT PATH"),
                Space::with_height(12),
                text_input(placeholder, &app.op_output)
                    .on_input(Msg::OpOutputChanged)
                    .style(|_, s| input_style(s))
                    .padding([10, 12])
                    .size(13),
            ]
            .spacing(0)
            .into(),
        )
    };

    // ── Preview ────────────────────────────────────────────────────────────
    let preview = container(
        row![
            // Big number
            container(
                column![text(affected.to_string()).size(48).style(tc(pal::ACCENT)),]
                    .align_x(alignment::Horizontal::Center),
            )
            .style(|_| container::Style {
                background: Some(tint(pal::ACCENT, 0.08).into()),
                border: iced::border::Border {
                    color: tint(pal::ACCENT, 0.25),
                    width: 1.0,
                    radius: 12.0.into(),
                },
                shadow: iced::Shadow {
                    color: tint(pal::ACCENT, 0.15),
                    offset: iced::Vector::new(0.0, 0.0),
                    blur_radius: 16.0,
                },
                ..Default::default()
            })
            .padding([14, 24]),
            Space::with_width(20),
            column![
                text("files will be affected").size(14).style(tc(pal::INK)),
                Space::with_height(4),
                row![
                    side_badge(app.op_side),
                    Space::with_width(6),
                    text("side selected").size(11).style(tc(pal::FAINT)),
                ]
                .align_y(alignment::Vertical::Center),
                Space::with_height(4),
                text(match app.op {
                    Operation::Zip => "Will be packed into a .zip archive",
                    Operation::Move => "Will be moved to the output folder",
                    Operation::Delete => "Will be permanently removed",
                    Operation::Export => "File names will be written to a list",
                })
                .size(11)
                .style(tc(pal::FAINT)),
            ]
            .spacing(0),
        ]
        .align_y(alignment::Vertical::Center),
    )
    .style(|_| container::Style {
        background: Some(pal::SURFACE.into()),
        border: iced::border::Border {
            color: pal::LINE,
            width: 1.0,
            radius: 12.0.into(),
        },
        ..Default::default()
    })
    .padding([20, 24])
    .width(Length::Fill);

    // ── Action button ──────────────────────────────────────────────────────
    let run_btn: Element<'_, Msg> = if app.op == Operation::Delete {
        btn_danger("Delete files")
            .on_press(Msg::RunOp)
            .width(Length::Fill)
            .into()
    } else {
        btn_primary(match app.op {
            Operation::Zip => "Create zip",
            Operation::Move => "Move files",
            Operation::Export => "Export list",
            Operation::Delete => unreachable!(),
        })
        .on_press(Msg::RunOp)
        .width(Length::Fill)
        .into()
    };

    column![
        pickers,
        Space::with_height(12),
        output_card,
        Space::with_height(12),
        preview,
        Space::with_height(14),
        run_btn,
    ]
    .spacing(0)
    .width(520)
    .into()
}

// ─────────────────────────────────────────────────────────────────────────────
// Root view
// ─────────────────────────────────────────────────────────────────────────────

pub fn view(app: &App) -> Element<'_, Msg> {
    let top_rule = container(Space::with_height(1))
        .style(|_| container::Style {
            background: Some(pal::LINE_DIM.into()),
            ..Default::default()
        })
        .width(Length::Fill);

    let bot_rule = container(Space::with_height(1))
        .style(|_| container::Style {
            background: Some(pal::LINE_DIM.into()),
            ..Default::default()
        })
        .width(Length::Fill);

    let panel: Element<'_, Msg> = match app.active_panel {
        Panel::Scan => view_scan(app),
        Panel::Operate => container(view_operate(app)).center_x(Length::Fill).into(),
    };

    let main = container(
        scrollable(container(panel).padding([28, 32]).width(Length::Fill)).height(Length::Fill),
    )
    .style(|_| container::Style {
        background: Some(pal::BG.into()),
        ..Default::default()
    })
    .width(Length::Fill)
    .height(Length::Fill);

    container(column![
        view_topbar(app),
        top_rule,
        main,
        bot_rule,
        view_statusbar(app),
    ])
    .style(|_| container::Style {
        background: Some(pal::BG.into()),
        ..Default::default()
    })
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
