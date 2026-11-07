//! Colour palettes and theme application.

use eframe::egui::{self, Color32, Rounding, Stroke, Visuals};
use crate::i18n::Strings;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ThemeKind {
    Classic,
    Light,
    Dark,
    Amoled,
}

impl ThemeKind {
    pub const ALL: [ThemeKind; 4] = [
        ThemeKind::Classic,
        ThemeKind::Light,
        ThemeKind::Dark,
        ThemeKind::Amoled,
    ];

    pub fn code(self) -> &'static str {
        match self {
            ThemeKind::Classic => "classic",
            ThemeKind::Light => "light",
            ThemeKind::Dark => "dark",
            ThemeKind::Amoled => "amoled",
        }
    }

    pub fn from_code(s: &str) -> Option<Self> {
        match s {
            "classic" => Some(ThemeKind::Classic),
            "light" => Some(ThemeKind::Light),
            "dark" => Some(ThemeKind::Dark),
            "amoled" => Some(ThemeKind::Amoled),
            _ => None,
        }
    }

    pub fn label(self, t: &Strings) -> &'static str {
        match self {
            ThemeKind::Classic => t.theme_classic,
            ThemeKind::Light => t.theme_light,
            ThemeKind::Dark => t.theme_dark,
            ThemeKind::Amoled => t.theme_amoled,
        }
    }

    pub fn palette(self) -> Palette {
        match self {
            ThemeKind::Classic => Palette {
                bg: Color32::from_rgb(0x1a, 0x1a, 0x1e),
                panel: Color32::from_rgb(0x23, 0x23, 0x29),
                widget: Color32::from_rgb(0x2d, 0x2d, 0x35),
                border: Color32::from_rgb(0x3a, 0x3a, 0x44),
                text: Color32::from_rgb(0xf0, 0xf0, 0xf4),
                muted: Color32::from_rgb(0x9a, 0x9a, 0xa6),
                accent: Color32::from_rgb(0xe0, 0x66, 0x2b),
                on_accent: Color32::from_rgb(0xff, 0xff, 0xff),
                warn: Color32::from_rgb(0xf0, 0xad, 0x4e),
                error: Color32::from_rgb(0xd9, 0x53, 0x4f),
                preview_bg: Color32::from_rgb(0x18, 0x18, 0x1b),
                preview_edge: Color32::from_rgb(0x40, 0x40, 0x48),
            },
            ThemeKind::Light => Palette {
                bg: Color32::from_rgb(0xf4, 0xf4, 0xf6),
                panel: Color32::from_rgb(0xff, 0xff, 0xff),
                widget: Color32::from_rgb(0xeb, 0xeb, 0xf0),
                border: Color32::from_rgb(0xd4, 0xd4, 0xdc),
                text: Color32::from_rgb(0x18, 0x18, 0x1c),
                muted: Color32::from_rgb(0x6e, 0x6e, 0x7a),
                accent: Color32::from_rgb(0xd4, 0x55, 0x1a),
                on_accent: Color32::from_rgb(0xff, 0xff, 0xff),
                warn: Color32::from_rgb(0xbd, 0x76, 0x00),
                error: Color32::from_rgb(0xc4, 0x2b, 0x26),
                preview_bg: Color32::from_rgb(0xe8, 0xe8, 0xed),
                preview_edge: Color32::from_rgb(0xbc, 0xbc, 0xc6),
            },
            ThemeKind::Dark => Palette {
                bg: Color32::from_rgb(0x12, 0x12, 0x14),
                panel: Color32::from_rgb(0x1c, 0x1c, 0x20),
                widget: Color32::from_rgb(0x26, 0x26, 0x2c),
                border: Color32::from_rgb(0x32, 0x32, 0x3a),
                text: Color32::from_rgb(0xee, 0xee, 0xf0),
                muted: Color32::from_rgb(0x88, 0x88, 0x94),
                accent: Color32::from_rgb(0xe0, 0x66, 0x2b),
                on_accent: Color32::from_rgb(0xff, 0xff, 0xff),
                warn: Color32::from_rgb(0xe0, 0x9e, 0x38),
                error: Color32::from_rgb(0xd9, 0x4e, 0x4a),
                preview_bg: Color32::from_rgb(0x10, 0x10, 0x12),
                preview_edge: Color32::from_rgb(0x38, 0x38, 0x40),
            },
            ThemeKind::Amoled => Palette {
                bg: Color32::from_rgb(0x00, 0x00, 0x00),
                panel: Color32::from_rgb(0x0c, 0x0c, 0x0e),
                widget: Color32::from_rgb(0x18, 0x18, 0x1c),
                border: Color32::from_rgb(0x28, 0x28, 0x30),
                text: Color32::from_rgb(0xf2, 0xf2, 0xf4),
                muted: Color32::from_rgb(0x82, 0x82, 0x8e),
                accent: Color32::from_rgb(0xe0, 0x66, 0x2b),
                on_accent: Color32::from_rgb(0xff, 0xff, 0xff),
                warn: Color32::from_rgb(0xe0, 0x9e, 0x38),
                error: Color32::from_rgb(0xd9, 0x4e, 0x4a),
                preview_bg: Color32::from_rgb(0x04, 0x04, 0x06),
                preview_edge: Color32::from_rgb(0x28, 0x28, 0x30),
            },
        }
    }
}

pub struct Palette {
    pub bg: Color32,
    pub panel: Color32,
    pub widget: Color32,
    pub border: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub accent: Color32,
    pub on_accent: Color32,
    pub warn: Color32,
    pub error: Color32,
    pub preview_bg: Color32,
    pub preview_edge: Color32,
}

pub fn apply(ctx: &egui::Context, kind: ThemeKind) {
    let p = kind.palette();
    let is_dark = kind != ThemeKind::Light;
    let mut vis = if is_dark { Visuals::dark() } else { Visuals::light() };

    vis.override_text_color = Some(p.text);
    vis.window_fill = p.panel;
    vis.panel_fill = p.panel;
    vis.window_rounding = Rounding::same(12.0);
    vis.window_stroke = Stroke::new(1.0_f32, p.border);

    vis.widgets.noninteractive.bg_fill = p.panel;
    vis.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, p.border);
    vis.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, p.muted);
    vis.widgets.noninteractive.rounding = Rounding::same(8.0);

    vis.widgets.inactive.bg_fill = p.widget;
    vis.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, p.border);
    vis.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, p.text);
    vis.widgets.inactive.rounding = Rounding::same(8.0);

    vis.widgets.hovered.bg_fill = p.widget;
    vis.widgets.hovered.bg_stroke = Stroke::new(1.2_f32, p.accent);
    vis.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, p.text);
    vis.widgets.hovered.rounding = Rounding::same(8.0);

    vis.widgets.active.bg_fill = p.accent;
    vis.widgets.active.bg_stroke = Stroke::new(1.0_f32, p.accent);
    vis.widgets.active.fg_stroke = Stroke::new(1.0_f32, p.on_accent);
    vis.widgets.active.rounding = Rounding::same(8.0);

    vis.widgets.open.bg_fill = p.widget;
    vis.widgets.open.bg_stroke = Stroke::new(1.0_f32, p.border);
    vis.widgets.open.fg_stroke = Stroke::new(1.0_f32, p.text);
    vis.widgets.open.rounding = Rounding::same(8.0);

    ctx.set_visuals(vis);
}