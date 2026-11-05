//! Four colour themes: classic (warm paper), light, dark and AMOLED black.

use eframe::egui::{self, Color32, CornerRadius, Stroke, Visuals};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeKind {
    Classic,
    Light,
    Dark,
    Amoled,
}

impl ThemeKind {
    pub const ALL: [ThemeKind; 4] = [ThemeKind::Classic, ThemeKind::Light, ThemeKind::Dark, ThemeKind::Amoled];

    pub fn code(self) -> &'static str {
        match self {
            ThemeKind::Classic => "classic",
            ThemeKind::Light => "light",
            ThemeKind::Dark => "dark",
            ThemeKind::Amoled => "amoled",
        }
    }
    pub fn from_code(s: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.code() == s)
    }
    pub fn label(self, t: &crate::i18n::Strings) -> &'static str {
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
                dark: false,
                bg: hex(0xfdf6ea),
                panel: hex(0xfffdf7),
                widget: hex(0xf4ead9),
                widget_hover: hex(0xeaddc6),
                border: hex(0xe5d9c3),
                text: hex(0x3a3127),
                muted: hex(0x8a7d6a),
                accent: hex(0xe0662b),
                accent_hover: hex(0xd96a35),
                on_accent: Color32::WHITE,
                warn: hex(0xb07d1a),
                error: hex(0xb3261e),
                preview_bg: hex(0xf8f0e0),
                preview_edge: hex(0x5a4a38),
            },
            ThemeKind::Light => Palette {
                dark: false,
                bg: hex(0xf3f4f6),
                panel: hex(0xffffff),
                widget: hex(0xeceef1),
                widget_hover: hex(0xdfe2e7),
                border: hex(0xd5d9df),
                text: hex(0x1f2328),
                muted: hex(0x6b7280),
                accent: hex(0xe0662b),
                accent_hover: hex(0xc9561f),
                on_accent: Color32::WHITE,
                warn: hex(0xa16207),
                error: hex(0xb91c1c),
                preview_bg: hex(0xf7f8fa),
                preview_edge: hex(0x30343a),
            },
            ThemeKind::Dark => Palette {
                dark: true,
                bg: hex(0x1b1c1f),
                panel: hex(0x232529),
                widget: hex(0x2e3035),
                widget_hover: hex(0x3a3d43),
                border: hex(0x3a3d43),
                text: hex(0xe6e6e6),
                muted: hex(0x9a9ca2),
                accent: hex(0xe07a48),
                accent_hover: hex(0xec8c5c),
                on_accent: hex(0x1b1c1f),
                warn: hex(0xd9a441),
                error: hex(0xef6b5e),
                preview_bg: hex(0x1f2024),
                preview_edge: hex(0x0c0c0e),
            },
            ThemeKind::Amoled => Palette {
                dark: true,
                bg: Color32::BLACK,
                panel: Color32::BLACK,
                widget: hex(0x121212),
                widget_hover: hex(0x1e1e1e),
                border: hex(0x262626),
                text: hex(0xeeeeee),
                muted: hex(0x8c8c8c),
                accent: hex(0xe07a48),
                accent_hover: hex(0xf08a58),
                on_accent: Color32::BLACK,
                warn: hex(0xd9a441),
                error: hex(0xff6b5e),
                preview_bg: Color32::BLACK,
                preview_edge: Color32::BLACK,
            },
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct Palette {
    pub dark: bool,
    pub bg: Color32,
    pub panel: Color32,
    pub widget: Color32,
    pub widget_hover: Color32,
    pub border: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub on_accent: Color32,
    pub warn: Color32,
    pub error: Color32,
    pub preview_bg: Color32,
    pub preview_edge: Color32,
}

const fn hex(v: u32) -> Color32 {
    Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
}

pub fn apply(ctx: &egui::Context, kind: ThemeKind) {
    let p = kind.palette();
    let mut v = if p.dark { Visuals::dark() } else { Visuals::light() };
    v.override_text_color = Some(p.text);
    v.weak_text_color = Some(p.muted);
    v.hyperlink_color = p.accent;
    v.faint_bg_color = p.widget;
    v.extreme_bg_color = p.widget;
    v.text_edit_bg_color = Some(p.widget);
    v.code_bg_color = p.widget;
    v.warn_fg_color = p.warn;
    v.error_fg_color = p.error;
    v.window_fill = p.panel;
    v.window_stroke = Stroke::new(1.0, p.border);
    v.panel_fill = p.bg;
    v.selection.bg_fill = p.accent;
    v.selection.stroke = Stroke::new(1.0, p.on_accent);
    v.slider_trailing_fill = true;
    let r = CornerRadius::same(8);
    let w = &mut v.widgets;
    w.noninteractive.bg_fill = p.panel;
    w.noninteractive.weak_bg_fill = p.panel;
    w.noninteractive.bg_stroke = Stroke::new(1.0, p.border);
    w.noninteractive.fg_stroke = Stroke::new(1.0, p.text);
    w.noninteractive.corner_radius = r;
    for (s, bg) in [(&mut w.inactive, p.widget), (&mut w.hovered, p.widget_hover), (&mut w.active, p.widget_hover), (&mut w.open, p.widget_hover)] {
        s.bg_fill = bg;
        s.weak_bg_fill = bg;
        s.corner_radius = r;
        s.fg_stroke = Stroke::new(1.0, p.text);
    }
    w.inactive.bg_stroke = Stroke::new(1.0, p.border);
    w.hovered.bg_stroke = Stroke::new(1.0, p.accent);
    w.active.bg_stroke = Stroke::new(1.5, p.accent);
    w.open.bg_stroke = Stroke::new(1.0, p.accent);
    ctx.set_theme(if p.dark { egui::Theme::Dark } else { egui::Theme::Light });
    ctx.set_visuals_of(if p.dark { egui::Theme::Dark } else { egui::Theme::Light }, v);
    ctx.style_mut_of(if p.dark { egui::Theme::Dark } else { egui::Theme::Light }, |s| {
        s.spacing.item_spacing = egui::vec2(8.0, 8.0);
        s.spacing.button_padding = egui::vec2(10.0, 6.0);
        s.spacing.interact_size.y = 30.0;
    });
}
