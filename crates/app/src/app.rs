//! The generator user interface.

use crate::i18n::{Lang, Strings};
use crate::preview::Preview;
use crate::theme::{self, Palette, ThemeKind};
use eframe::egui::{self, Color32, Margin, RichText, Rounding, Stroke};
use generator_3mf_core::gridfinity::{
    BatteryKind, ToolHoleKind, DIV_RANGE, HEIGHT_RANGE, UNITS_RANGE, WALL_RANGE,
};
use generator_3mf_core::{export, generate, Kind, Mesh, Params, Warning};

#[allow(dead_code)]
type Job = (Params, Mesh, Vec<Warning>);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ActiveTab {
    Bins,
    Tools,
    Batteries,
    Baseplates,
    Lids,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Preset {
    Custom,
    Small,
    Workhorse,
    Tray,
    Organizer,
    Tall,
    Plate3,
    Plate4,
}

impl Preset {
    const ALL: [Preset; 7] = [
        Preset::Small,
        Preset::Workhorse,
        Preset::Tray,
        Preset::Organizer,
        Preset::Tall,
        Preset::Plate3,
        Preset::Plate4,
    ];
    fn label(self, t: &Strings) -> &'static str {
        match self {
            Preset::Custom => t.custom,
            Preset::Small => t.preset_small,
            Preset::Workhorse => t.preset_workhorse,
            Preset::Tray => t.preset_tray,
            Preset::Organizer => t.preset_organizer,
            Preset::Tall => t.preset_tall,
            Preset::Plate3 => t.preset_plate3,
            Preset::Plate4 => t.preset_plate4,
        }
    }
    fn apply(self, p: &mut Params) {
        let bin = |ux, uy, h, dx, dy| Params {
            kind: Kind::Bin,
            units_x: ux,
            units_y: uy,
            height_u: h,
            div_x: dx,
            div_y: dy,
            ..p.clone()
        };
        *p = match self {
            Preset::Custom => return,
            Preset::Small => bin(1, 1, 3, 1, 1),
            Preset::Workhorse => bin(2, 1, 6, 1, 1),
            Preset::Tray => bin(2, 2, 3, 2, 2),
            Preset::Organizer => bin(3, 2, 6, 3, 2),
            Preset::Tall => bin(1, 2, 12, 1, 1),
            Preset::Plate3 => Params {
                kind: Kind::Baseplate,
                units_x: 3,
                units_y: 3,
                ..p.clone()
            },
            Preset::Plate4 => Params {
                kind: Kind::Baseplate,
                units_x: 4,
                units_y: 4,
                ..p.clone()
            },
        };
    }
}

pub struct GeneratorApp {
    tab: ActiveTab,
    params: Params,
    name: String,
    color: [u8; 3],
    lang: Lang,
    theme: ThemeKind,
    applied_theme: Option<ThemeKind>,
    preset: Preset,
    mesh: Option<Mesh>,
    built: Option<Params>,
    warnings: Vec<Warning>,
    preview: Preview,
    status: Option<(String, bool)>,
    #[cfg(not(target_arch = "wasm32"))]
    job: Option<std::sync::mpsc::Receiver<Job>>,
    #[cfg(target_arch = "wasm32")]
    job: Option<()>,
}

const DEFAULT_COLOR: [u8; 3] = [0xe0, 0x66, 0x2b];

fn detect_lang() -> Lang {
    sys_locale::get_locale()
        .map(|l| {
            if l.to_lowercase().starts_with("ru") {
                Lang::Ru
            } else {
                Lang::En
            }
        })
        .unwrap_or(Lang::En)
}

fn params_to_string(p: &Params) -> String {
    let k_str = match p.kind {
        Kind::Bin => "bin",
        Kind::ToolHolder => "tool",
        Kind::BatteryTray => "battery",
        Kind::Lid => "lid",
        Kind::Baseplate => "plate",
    };
    format!(
        "{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{};{}",
        k_str,
        p.units_x,
        p.units_y,
        p.height_u,
        p.div_x,
        p.div_y,
        p.lip as u8,
        p.magnets as u8,
        p.scoop as u8,
        p.label as u8,
        p.screws as u8,
        p.wall,
        p.plate_magnets as u8,
        p.tool_holes_x,
        p.tool_holes_y,
        match p.tool_hole_kind {
            ToolHoleKind::HexBit => 0,
            ToolHoleKind::Round => 1,
        },
        p.tool_hole_depth,
        p.tool_hole_size,
        match p.battery_kind {
            BatteryKind::AA => 0,
            BatteryKind::AAA => 1,
            BatteryKind::B18650 => 2,
            BatteryKind::B21700 => 3,
            BatteryKind::Custom => 4,
        },
        p.battery_holes_x,
        p.battery_holes_y,
        p.battery_hole_depth,
        p.battery_hole_dia,
        p.plate_skeleton as u8,
        p.lid_finger_scoop as u8,
    )
}

fn params_from_string(s: &str) -> Option<Params> {
    let f: Vec<&str> = s.split(';').collect();
    if f.len() < 13 {
        return None;
    }
    let b = |i: usize| f.get(i).map(|&v| v == "1").unwrap_or(false);
    let n = |i: usize| f[i].parse::<u32>().ok();
    let kind = match f[0] {
        "tool" => Kind::ToolHolder,
        "battery" => Kind::BatteryTray,
        "lid" => Kind::Lid,
        "plate" => Kind::Baseplate,
        _ => Kind::Bin,
    };
    let tool_holes_x = f.get(13).and_then(|s| s.parse().ok()).unwrap_or(5);
    let tool_holes_y = f.get(14).and_then(|s| s.parse().ok()).unwrap_or(2);
    let tool_hole_kind = if f.get(15).map(|&s| s == "1").unwrap_or(false) {
        ToolHoleKind::Round
    } else {
        ToolHoleKind::HexBit
    };
    let tool_hole_depth = f.get(16).and_then(|s| s.parse().ok()).unwrap_or(10.0);
    let tool_hole_size = f.get(17).and_then(|s| s.parse().ok()).unwrap_or(6.8);

    let battery_kind = match f.get(18).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0) {
        1 => BatteryKind::AAA,
        2 => BatteryKind::B18650,
        3 => BatteryKind::B21700,
        4 => BatteryKind::Custom,
        _ => BatteryKind::AA,
    };
    let battery_holes_x = f.get(19).and_then(|s| s.parse().ok()).unwrap_or(4);
    let battery_holes_y = f.get(20).and_then(|s| s.parse().ok()).unwrap_or(2);
    let battery_hole_depth = f.get(21).and_then(|s| s.parse().ok()).unwrap_or(25.0);
    let battery_hole_dia = f.get(22).and_then(|s| s.parse().ok()).unwrap_or(15.0);
    let plate_skeleton = b(23);
    let lid_finger_scoop = if f.len() > 24 { b(24) } else { true };

    Some(
        Params {
            kind,
            units_x: n(1)?,
            units_y: n(2)?,
            height_u: n(3)?,
            div_x: n(4)?,
            div_y: n(5)?,
            lip: b(6),
            magnets: b(7),
            scoop: b(8),
            label: b(9),
            screws: b(10),
            wall: f[11].parse().ok()?,
            plate_magnets: b(12),
            plate_skeleton,
            tool_holes_x,
            tool_holes_y,
            tool_hole_kind,
            tool_hole_depth,
            tool_hole_size,
            battery_kind,
            battery_holes_x,
            battery_holes_y,
            battery_hole_depth,
            battery_hole_dia,
            lid_finger_scoop,
        }
        .clamped(),
    )
}

fn color_hex(c: [u8; 3]) -> String {
    format!("{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

fn color_from_hex(s: &str) -> Option<[u8; 3]> {
    if s.len() != 6 {
        return None;
    }
    let v = u32::from_str_radix(s, 16).ok()?;
    Some([(v >> 16) as u8, (v >> 8) as u8, v as u8])
}

impl GeneratorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self {
            tab: ActiveTab::Bins,
            params: Params::default(),
            name: "generator".into(),
            color: DEFAULT_COLOR,
            lang: detect_lang(),
            theme: ThemeKind::Classic,
            applied_theme: None,
            preset: Preset::Custom,
            mesh: None,
            built: None,
            warnings: Vec::new(),
            preview: Preview::default(),
            status: None,
            job: None,
        };
        if let Some(s) = cc.storage {
            if let Some(l) = s.get_string("lang").and_then(|v| Lang::from_code(&v)) {
                app.lang = l;
            }
            if let Some(t) = s.get_string("theme").and_then(|v| ThemeKind::from_code(&v)) {
                app.theme = t;
            }
            if let Some(c) = s.get_string("color").and_then(|v| color_from_hex(&v)) {
                app.color = c;
            }
            if let Some(n) = s.get_string("name") {
                if !n.trim().is_empty() {
                    app.name = n;
                }
            }
            if let Some(p) = s.get_string("params").and_then(|v| params_from_string(&v)) {
                app.tab = match p.kind {
                    Kind::Bin => ActiveTab::Bins,
                    Kind::ToolHolder => ActiveTab::Tools,
                    Kind::BatteryTray => ActiveTab::Batteries,
                    Kind::Lid => ActiveTab::Lids,
                    Kind::Baseplate => ActiveTab::Baseplates,
                };
                app.params = p;
            }
        }
        app
    }

    fn file_stem(&self) -> String {
        let clean: String = self
            .name
            .trim()
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let base = if clean.is_empty() {
            "generator".to_string()
        } else {
            clean
        };
        format!("{base}_{}", self.params.slug())
    }

    fn poll_generation(&mut self, ctx: &egui::Context) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(rx) = &self.job {
            match rx.try_recv() {
                Ok((p, m, w)) => {
                    self.preview.set_mesh(&m);
                    self.mesh = Some(m);
                    self.built = Some(p);
                    self.warnings = w;
                    self.job = None;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(std::time::Duration::from_millis(30))
                }
                Err(_) => self.job = None,
            }
        }
        let wanted = self.params.clamped();
        if self.job.is_some() || self.built.as_ref() == Some(&wanted) {
            return;
        }
        if ctx.input(|i| i.pointer.any_down()) && self.mesh.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(60));
            return;
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let (tx, rx) = std::sync::mpsc::channel();
            let ctx2 = ctx.clone();
            std::thread::spawn(move || {
                let (m, w) = generate(&wanted);
                let _ = tx.send((wanted, m, w));
                ctx2.request_repaint();
            });
            self.job = Some(rx);
        }
        #[cfg(target_arch = "wasm32")]
        {
            let (m, w) = generate(&wanted);
            self.preview.set_mesh(&m);
            self.mesh = Some(m);
            self.built = Some(wanted);
            self.warnings = w;
        }
    }

    fn export(&mut self, three_mf: bool) {
        let Some(mesh) = &self.mesh else { return };
        let t = self.lang.t();
        let stem = self.file_stem();
        let (bytes, file, mime) = if three_mf {
            (
                export::three_mf(mesh, &stem, self.color, [128.0, 128.0]),
                format!("{stem}.3mf"),
                "model/3mf",
            )
        } else {
            (export::stl(mesh, &stem), format!("{stem}.stl"), "model/stl")
        };
        self.status = match crate::save::save(&bytes, &file, mime) {
            Ok(Some(loc)) => Some((format!("{}: {loc}", t.saved_to), false)),
            Ok(None) => None,
            Err(e) => Some((format!("{}: {e}", t.save_failed), true)),
        };
    }
}

fn card<R>(
    ui: &mut egui::Ui,
    pal: &Palette,
    title: &str,
    add: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    egui::Frame::none()
        .fill(pal.panel)
        .stroke(Stroke::new(1.0_f32, pal.border))
        .rounding(Rounding::same(12.0))
        .inner_margin(Margin::same(14.0))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(title).strong().size(16.0));
            ui.add_space(4.0);
            add(ui)
        })
        .inner
}

fn stepper(
    ui: &mut egui::Ui,
    label: &str,
    hint: Option<&str>,
    v: &mut u32,
    range: (u32, u32),
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        let r = ui.label(label);
        if let Some(h) = hint {
            r.on_hover_text(h);
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .add_enabled(
                    *v < range.1,
                    egui::Button::new(RichText::new("+").strong()).min_size(egui::vec2(32.0, 30.0)),
                )
                .clicked()
            {
                *v += 1;
                changed = true;
            }
            if ui
                .add(egui::DragValue::new(v).range(range.0..=range.1).speed(0.1))
                .changed()
            {
                changed = true;
            }
            if ui
                .add_enabled(
                    *v > range.0,
                    egui::Button::new(RichText::new("−").strong()).min_size(egui::vec2(32.0, 30.0)),
                )
                .clicked()
            {
                *v -= 1;
                changed = true;
            }
        });
    });
    changed
}

fn toggle(ui: &mut egui::Ui, pal: &Palette, label: &str, hint: &str, v: &mut bool) -> bool {
    let c = ui.checkbox(v, label).changed();
    ui.indent(label, |ui| {
        ui.label(RichText::new(hint).size(12.5).color(pal.muted));
    });
    c
}

impl GeneratorApp {
    fn header(&mut self, ui: &mut egui::Ui, pal: &Palette, narrow: bool) {
        let t = self.lang.t();
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(egui::vec2(30.0, 30.0), egui::Sense::hover());
            paint_logo(ui.painter(), rect, pal.accent, pal.on_accent);
            let width = ui.available_width();
            if width > 480.0 {
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 0.0;
                    ui.label(RichText::new(t.app_title).strong().size(19.0));
                    if !narrow {
                        ui.label(RichText::new(t.subtitle).size(12.5).color(pal.muted));
                    }
                });
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                for l in Lang::ALL.iter().rev() {
                    let on = self.lang == *l;
                    let b =
                        egui::Button::new(RichText::new(l.code().to_uppercase()).color(if on {
                            pal.on_accent
                        } else {
                            pal.text
                        }))
                        .fill(if on { pal.accent } else { pal.widget })
                        .min_size(egui::vec2(38.0, 28.0));
                    if ui.add(b).on_hover_text(l.native_name()).clicked() {
                        self.lang = *l;
                    }
                }
                ui.add_space(6.0);
                egui::ComboBox::from_id_salt("theme")
                    .selected_text(if narrow {
                        self.theme.label(t).to_string()
                    } else {
                        format!("{}: {}", t.theme, self.theme.label(t))
                    })
                    .show_ui(ui, |ui| {
                        for k in ThemeKind::ALL {
                            ui.selectable_value(&mut self.theme, k, k.label(t));
                        }
                    });
            });
        });
    }

    fn controls(&mut self, ui: &mut egui::Ui, pal: &Palette) {
        let t = self.lang.t();
        let mut changed = false;

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;
            let tabs = [
                (ActiveTab::Bins, t.tab_bins),
                (ActiveTab::Tools, t.tab_tools),
                (ActiveTab::Batteries, t.tab_batteries),
                (ActiveTab::Baseplates, t.tab_baseplates),
                (ActiveTab::Lids, t.tab_lids),
            ];
            for (tab, label) in tabs {
                let is_active = self.tab == tab;
                let btn = egui::Button::new(RichText::new(label).strong().color(if is_active {
                    pal.on_accent
                } else {
                    pal.text
                }))
                .fill(if is_active { pal.accent } else { pal.widget })
                .min_size(egui::vec2(60.0, 30.0));

                if ui.add(btn).clicked() {
                    self.tab = tab;
                    match tab {
                        ActiveTab::Bins => self.params.kind = Kind::Bin,
                        ActiveTab::Tools => self.params.kind = Kind::ToolHolder,
                        ActiveTab::Batteries => self.params.kind = Kind::BatteryTray,
                        ActiveTab::Baseplates => self.params.kind = Kind::Baseplate,
                        ActiveTab::Lids => self.params.kind = Kind::Lid,
                    }
                    changed = true;
                }
            }
        });
        ui.add_space(8.0);

        card(ui, pal, t.file, |ui| {
            egui::Grid::new("file_grid")
                .num_columns(2)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    ui.label(t.name);
                    ui.add(
                        egui::TextEdit::singleline(&mut self.name)
                            .char_limit(48)
                            .desired_width(f32::INFINITY),
                    );
                    ui.end_row();
                    ui.label(t.preset);
                    let mut sel = self.preset;
                    egui::ComboBox::from_id_salt("preset")
                        .width(ui.available_width())
                        .selected_text(self.preset.label(t))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut sel, Preset::Custom, t.custom);
                            for p in Preset::ALL {
                                ui.selectable_value(&mut sel, p, p.label(t));
                            }
                        });
                    if sel != self.preset {
                        self.preset = sel;
                        sel.apply(&mut self.params);
                        changed = true;
                    }
                    ui.end_row();
                });
        });
        ui.add_space(8.0);

        match self.tab {
            ActiveTab::Bins => {
                card(ui, pal, t.part, |ui| {
                    changed |= stepper(
                        ui,
                        t.units_wide,
                        Some(t.units_hint),
                        &mut self.params.units_x,
                        UNITS_RANGE,
                    );
                    changed |= stepper(
                        ui,
                        t.units_deep,
                        Some(t.units_hint),
                        &mut self.params.units_y,
                        UNITS_RANGE,
                    );
                    changed |= stepper(
                        ui,
                        t.height_u,
                        Some(t.height_hint),
                        &mut self.params.height_u,
                        HEIGHT_RANGE,
                    );
                    ui.label(RichText::new(t.height_hint).size(12.5).color(pal.muted));
                });
                ui.add_space(8.0);

                card(ui, pal, t.bin_features, |ui| {
                    changed |= stepper(ui, t.comps_across, None, &mut self.params.div_x, DIV_RANGE);
                    changed |= stepper(ui, t.comps_deep, None, &mut self.params.div_y, DIV_RANGE);
                    ui.add_space(4.0);
                    changed |= toggle(ui, pal, t.lip, t.lip_hint, &mut self.params.lip);
                    changed |= toggle(ui, pal, t.magnets, t.magnets_hint, &mut self.params.magnets);
                    changed |= toggle(ui, pal, t.scoop, t.scoop_hint, &mut self.params.scoop);
                    changed |= toggle(ui, pal, t.label, t.label_hint, &mut self.params.label);
                    changed |= toggle(ui, pal, t.screws, t.screws_hint, &mut self.params.screws);
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(t.wall);
                        changed |= ui
                            .add(
                                egui::Slider::new(
                                    &mut self.params.wall,
                                    WALL_RANGE.0..=WALL_RANGE.1,
                                )
                                .step_by(0.1)
                                .fixed_decimals(1),
                            )
                            .changed();
                    });
                    ui.horizontal(|ui| {
                        ui.label(t.color);
                        ui.color_edit_button_srgb(&mut self.color);
                    });
                });
                ui.add_space(8.0);
            }

            ActiveTab::Tools => {
                card(ui, pal, t.part, |ui| {
                    changed |= stepper(
                        ui,
                        t.units_wide,
                        Some(t.units_hint),
                        &mut self.params.units_x,
                        UNITS_RANGE,
                    );
                    changed |= stepper(
                        ui,
                        t.units_deep,
                        Some(t.units_hint),
                        &mut self.params.units_y,
                        UNITS_RANGE,
                    );
                    changed |= stepper(
                        ui,
                        t.height_u,
                        Some(t.height_hint),
                        &mut self.params.height_u,
                        (3, 10),
                    );
                    ui.label(RichText::new(t.height_hint).size(12.5).color(pal.muted));
                });
                ui.add_space(8.0);

                card(ui, pal, t.tool_features, |ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(
                                self.params.tool_hole_kind == ToolHoleKind::HexBit,
                                t.tool_kind_hex,
                            )
                            .clicked()
                        {
                            self.params.tool_hole_kind = ToolHoleKind::HexBit;
                            self.params.tool_hole_size = 6.8;
                            changed = true;
                        }
                        if ui
                            .selectable_label(
                                self.params.tool_hole_kind == ToolHoleKind::Round,
                                t.tool_kind_round,
                            )
                            .clicked()
                        {
                            self.params.tool_hole_kind = ToolHoleKind::Round;
                            self.params.tool_hole_size = 4.0;
                            changed = true;
                        }
                    });
                    ui.add_space(4.0);
                    changed |= stepper(
                        ui,
                        t.tool_holes_across,
                        None,
                        &mut self.params.tool_holes_x,
                        (1, 16),
                    );
                    changed |= stepper(
                        ui,
                        t.tool_holes_deep,
                        None,
                        &mut self.params.tool_holes_y,
                        (1, 16),
                    );
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(t.tool_hole_depth);
                        changed |= ui
                            .add(
                                egui::Slider::new(&mut self.params.tool_hole_depth, 3.0..=35.0)
                                    .step_by(0.5)
                                    .fixed_decimals(1),
                            )
                            .changed();
                    });
                    ui.horizontal(|ui| {
                        let r = ui.label(t.tool_hole_size);
                        r.on_hover_text(t.tool_hole_size_hint);
                        changed |= ui
                            .add(
                                egui::Slider::new(&mut self.params.tool_hole_size, 2.0..=15.0)
                                    .step_by(0.1)
                                    .fixed_decimals(1),
                            )
                            .changed();
                    });
                    ui.add_space(4.0);
                    changed |= toggle(ui, pal, t.magnets, t.magnets_hint, &mut self.params.magnets);
                    changed |= toggle(ui, pal, t.screws, t.screws_hint, &mut self.params.screws);
                    ui.horizontal(|ui| {
                        ui.label(t.color);
                        ui.color_edit_button_srgb(&mut self.color);
                    });
                });
                ui.add_space(8.0);
            }

            ActiveTab::Batteries => {
                card(ui, pal, t.part, |ui| {
                    changed |= stepper(
                        ui,
                        t.units_wide,
                        Some(t.units_hint),
                        &mut self.params.units_x,
                        UNITS_RANGE,
                    );
                    changed |= stepper(
                        ui,
                        t.units_deep,
                        Some(t.units_hint),
                        &mut self.params.units_y,
                        UNITS_RANGE,
                    );
                    changed |= stepper(
                        ui,
                        t.height_u,
                        Some(t.height_hint),
                        &mut self.params.height_u,
                        (3, 14),
                    );
                    ui.label(RichText::new(t.height_hint).size(12.5).color(pal.muted));
                });
                ui.add_space(8.0);

                card(ui, pal, t.battery_features, |ui| {
                    ui.label(RichText::new(t.battery_type).color(pal.muted));
                    ui.horizontal_wrapped(|ui| {
                        let types = [
                            (BatteryKind::AA, "AA (Ø15)", 15.0, 25.0, 5),
                            (BatteryKind::AAA, "AAA (Ø11.2)", 11.2, 20.0, 4),
                            (BatteryKind::B18650, "18650 (Ø19.2)", 19.2, 35.0, 6),
                            (BatteryKind::B21700, "21700 (Ø22.4)", 22.4, 40.0, 7),
                            (
                                BatteryKind::Custom,
                                "Свой",
                                self.params.battery_hole_dia,
                                self.params.battery_hole_depth,
                                self.params.height_u,
                            ),
                        ];
                        for (k, label, dia, depth, def_h) in types {
                            let on = self.params.battery_kind == k;
                            if ui.selectable_label(on, label).clicked() {
                                self.params.battery_kind = k;
                                if k != BatteryKind::Custom {
                                    self.params.battery_hole_dia = dia;
                                    self.params.battery_hole_depth = depth;
                                    self.params.height_u = self.params.height_u.max(def_h);
                                }
                                changed = true;
                            }
                        }
                    });
                    ui.add_space(4.0);
                    changed |= stepper(
                        ui,
                        t.battery_holes_across,
                        None,
                        &mut self.params.battery_holes_x,
                        (1, 16),
                    );
                    changed |= stepper(
                        ui,
                        t.battery_holes_deep,
                        None,
                        &mut self.params.battery_holes_y,
                        (1, 16),
                    );
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(t.battery_hole_depth);
                        changed |= ui
                            .add(
                                egui::Slider::new(&mut self.params.battery_hole_depth, 5.0..=65.0)
                                    .step_by(0.5)
                                    .fixed_decimals(1),
                            )
                            .changed();
                    });
                    ui.horizontal(|ui| {
                        ui.label(t.battery_hole_dia);
                        changed |= ui
                            .add(
                                egui::Slider::new(&mut self.params.battery_hole_dia, 6.0..=35.0)
                                    .step_by(0.1)
                                    .fixed_decimals(1),
                            )
                            .changed();
                    });
                    ui.add_space(4.0);
                    changed |= toggle(ui, pal, t.magnets, t.magnets_hint, &mut self.params.magnets);
                    changed |= toggle(ui, pal, t.screws, t.screws_hint, &mut self.params.screws);
                    ui.horizontal(|ui| {
                        ui.label(t.color);
                        ui.color_edit_button_srgb(&mut self.color);
                    });
                });
                ui.add_space(8.0);
            }

            ActiveTab::Baseplates => {
                card(ui, pal, t.part, |ui| {
                    changed |= stepper(
                        ui,
                        t.units_wide,
                        Some(t.units_hint),
                        &mut self.params.units_x,
                        UNITS_RANGE,
                    );
                    changed |= stepper(
                        ui,
                        t.units_deep,
                        Some(t.units_hint),
                        &mut self.params.units_y,
                        UNITS_RANGE,
                    );
                    ui.label(RichText::new(t.units_hint).size(12.5).color(pal.muted));
                });
                ui.add_space(8.0);

                card(ui, pal, t.plate_features, |ui| {
                    changed |= toggle(
                        ui,
                        pal,
                        t.plate_skeleton,
                        t.plate_skeleton_hint,
                        &mut self.params.plate_skeleton,
                    );
                    changed |= toggle(
                        ui,
                        pal,
                        t.magnets,
                        t.plate_magnets_hint,
                        &mut self.params.plate_magnets,
                    );
                    ui.horizontal(|ui| {
                        ui.label(t.color);
                        ui.color_edit_button_srgb(&mut self.color);
                    });
                });
                ui.add_space(8.0);
            }

            ActiveTab::Lids => {
                card(ui, pal, t.part, |ui| {
                    changed |= stepper(
                        ui,
                        t.units_wide,
                        Some(t.units_hint),
                        &mut self.params.units_x,
                        UNITS_RANGE,
                    );
                    changed |= stepper(
                        ui,
                        t.units_deep,
                        Some(t.units_hint),
                        &mut self.params.units_y,
                        UNITS_RANGE,
                    );
                    ui.label(RichText::new(t.units_hint).size(12.5).color(pal.muted));
                });
                ui.add_space(8.0);

                card(ui, pal, t.lid_features, |ui| {
                    changed |= toggle(
                        ui,
                        pal,
                        t.lid_finger_scoop,
                        t.lid_finger_scoop_hint,
                        &mut self.params.lid_finger_scoop,
                    );
                    ui.horizontal(|ui| {
                        ui.label(t.color);
                        ui.color_edit_button_srgb(&mut self.color);
                    });
                });
                ui.add_space(8.0);
            }
        }

        card(ui, pal, t.export, |ui| {
            let ready = self.mesh.is_some()
                && self.job.is_none()
                && self.built.as_ref() == Some(&self.params.clamped());
            ui.horizontal(|ui| {
                let w = (ui.available_width() - 8.0) / 2.0;
                let primary =
                    egui::Button::new(RichText::new(t.save_3mf).strong().color(pal.on_accent))
                        .fill(pal.accent)
                        .min_size(egui::vec2(w, 38.0));
                if ui.add_enabled(ready, primary).clicked() {
                    self.export(true);
                }
                let secondary = egui::Button::new(t.save_stl).min_size(egui::vec2(w, 38.0));
                if ui.add_enabled(ready, secondary).clicked() {
                    self.export(false);
                }
            });
            if let Some((msg, err)) = &self.status {
                ui.label(RichText::new(msg).size(12.5).color(if *err {
                    pal.error
                } else {
                    pal.muted
                }));
            }
        });
        ui.add_space(8.0);

        card(ui, pal, t.notes_title, |ui| {
            for n in t.notes {
                ui.label(RichText::new(format!("• {n}")).size(13.0).color(pal.muted));
            }
        });
        ui.add_space(4.0);
        ui.label(
            RichText::new(format!("generator_3mf v{}", generator_3mf_core::VERSION))
                .size(11.0)
                .color(pal.muted),
        );

        if changed {
            self.preset = Preset::Custom;
            self.status = None;
        }
    }

    fn preview_area(&mut self, ui: &mut egui::Ui, pal: &Palette) {
        let t = self.lang.t();
        let rect = ui.available_rect_before_wrap();
        self.preview.show(ui, rect, pal, self.color);
        let painter = ui.painter_at(rect);
        // Stats overlay.
        let p = self.built.clone().unwrap_or_else(|| self.params.clamped());
        let s = p.size();
        let mut lines = vec![format!(
            "{}: {:.1} × {:.1} × {:.1} mm",
            t.size, s[0], s[1], s[2]
        )];
        match p.kind {
            Kind::Bin => {
                lines.push(format!("{}: {} × {}", t.compartments, p.div_x, p.div_y));
            }
            Kind::ToolHolder => {
                let k_desc = match p.tool_hole_kind {
                    ToolHoleKind::HexBit => "1/4\"",
                    ToolHoleKind::Round => "Ø",
                };
                lines.push(format!(
                    "Отсеки: {} × {} ({})",
                    p.tool_holes_x, p.tool_holes_y, k_desc
                ));
            }
            Kind::BatteryTray => {
                lines.push(format!(
                    "{}: {} × {} (Ø{:.1} мм)",
                    t.compartments, p.battery_holes_x, p.battery_holes_y, p.battery_hole_dia
                ));
            }
            Kind::Lid => {
                lines.push(format!("Крышка {}×{}", p.units_x, p.units_y));
            }
            Kind::Baseplate => {
                if p.plate_skeleton {
                    lines.push("Скелетное основание".into());
                }
            }
        }
        if let Some(m) = &self.mesh {
            lines.push(format!("{}: {}", t.triangles, m.triangles.len()));
            lines.push(format!("{}: {:.1} cm³", t.volume, m.volume() / 1000.0));
        }
        let mut y = rect.top() + 12.0;
        let kind = match p.kind {
            Kind::Bin => t.bin,
            Kind::ToolHolder => t.tool_holder,
            Kind::BatteryTray => t.battery_tray,
            Kind::Lid => t.lid,
            Kind::Baseplate => t.baseplate,
        };
        painter.text(
            egui::pos2(rect.left() + 14.0, y),
            egui::Align2::LEFT_TOP,
            kind,
            egui::FontId::proportional(16.0),
            pal.text,
        );
        y += 22.0;
        for l in &lines {
            painter.text(
                egui::pos2(rect.left() + 14.0, y),
                egui::Align2::LEFT_TOP,
                l,
                egui::FontId::proportional(13.0),
                pal.muted,
            );
            y += 18.0;
        }
        let mut wy = rect.bottom() - 30.0;
        for w in self.warnings.iter().rev() {
            let msg = match w {
                Warning::CompartmentsTooSmall => t.warn_comps,
                Warning::LabelSkipped => t.warn_label,
                Warning::ScoopReduced => t.warn_scoop,
            };
            if *w == Warning::ScoopReduced {
                continue;
            }
            painter.text(
                egui::pos2(rect.left() + 14.0, wy),
                egui::Align2::LEFT_BOTTOM,
                format!("⚠ {msg}"),
                egui::FontId::proportional(13.0),
                pal.warn,
            );
            wy -= 18.0;
        }
        painter.text(
            egui::pos2(rect.left() + 14.0, rect.bottom() - 10.0),
            egui::Align2::LEFT_BOTTOM,
            t.preview_hint,
            egui::FontId::proportional(12.0),
            pal.muted,
        );
        if self.job.is_some() || self.mesh.is_none() {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                t.building,
                egui::FontId::proportional(15.0),
                pal.muted,
            );
        }
        // Reset button.
        let bw = 24.0 + 7.5 * t.reset_view.chars().count() as f32;
        let br = egui::Rect::from_min_size(
            egui::pos2(rect.right() - bw - 10.0, rect.top() + 10.0),
            egui::vec2(bw, 30.0),
        );
        if ui
            .put(
                br,
                egui::Button::new(RichText::new(t.reset_view).size(12.5)),
            )
            .clicked()
        {
            self.preview.reset_view();
        }
    }
}

fn paint_logo(p: &egui::Painter, r: egui::Rect, bg: Color32, fg: Color32) {
    p.rect_filled(r, 7.0, bg);
    let inner = r.shrink(7.0);
    let cell = inner.width() / 2.0;
    for i in 0..2 {
        for j in 0..2 {
            let c = egui::Rect::from_min_size(
                inner.min + egui::vec2(i as f32 * cell, j as f32 * cell),
                egui::vec2(cell, cell),
            )
            .shrink(1.2);
            p.rect_filled(c, 2.0, fg);
        }
    }
}

impl eframe::App for GeneratorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(target_os = "android")]
        egui::TopBottomPanel::top("android_safe_area_padding")
            .exact_height(42.0)
            .show_separator_line(false)
            .frame(egui::Frame::none().fill(ctx.style().visuals.panel_fill))
            .show(ctx, |_ui| {});
        if self.applied_theme != Some(self.theme) {
            theme::apply(ctx, self.theme);
            self.applied_theme = Some(self.theme);
        }
        let pal = self.theme.palette();
        self.poll_generation(ctx);

        let screen = ctx.screen_rect();
        let narrow = screen.width() < 760.0;
        let bar = egui::Frame::none()
            .fill(pal.bg)
            .inner_margin(Margin::symmetric(14.0, 10.0));
        egui::TopBottomPanel::top("header")
            .frame(bar)
            .show(ctx, |ui| self.header(ui, &pal, narrow));

        let body = egui::Frame::none().fill(pal.bg).inner_margin(Margin {
            left: 14.0,
            right: 14.0,
            top: 0.0,
            bottom: 14.0,
        });
        if narrow {
            let h = (screen.height() * 0.42).clamp(220.0, 520.0);
            egui::TopBottomPanel::top("preview_mobile")
                .frame(body)
                .exact_height(h)
                .resizable(false)
                .show(ctx, |ui| self.preview_area(ui, &pal));
            egui::CentralPanel::default().frame(body).show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.controls(ui, &pal));
            });
        } else {
            egui::SidePanel::left("controls")
                .frame(body)
                .resizable(true)
                .default_width(380.0)
                .min_width(320.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.add_space(2.0);
                        self.controls(ui, &pal);
                    });
                });
            egui::CentralPanel::default()
                .frame(body)
                .show(ctx, |ui| self.preview_area(ui, &pal));
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("lang", self.lang.code().into());
        storage.set_string("theme", self.theme.code().into());
        storage.set_string("color", color_hex(self.color));
        storage.set_string("name", self.name.clone());
        storage.set_string("params", params_to_string(&self.params));
    }
}

#[allow(dead_code)]
/// Procedural application icon (orange tile with a 2×2 grid).
pub fn icon() -> egui::IconData {
    let n = 64usize;
    let mut rgba = vec![0u8; n * n * 4];
    let inside = |x: f32, y: f32, x0: f32, y0: f32, x1: f32, y1: f32, r: f32| {
        let cx = x.clamp(x0 + r, x1 - r);
        let cy = y.clamp(y0 + r, y1 - r);
        (x - cx).powi(2) + (y - cy).powi(2) <= r * r
    };
    for y in 0..n {
        for x in 0..n {
            let (fx, fy) = (x as f32 + 0.5, y as f32 + 0.5);
            let i = (y * n + x) * 4;
            if inside(fx, fy, 2.0, 2.0, 62.0, 62.0, 14.0) {
                let mut c = [0xe0u8, 0x66, 0x2b, 255];
                let cells = [(14.0, 14.0), (33.0, 14.0), (14.0, 33.0), (33.0, 33.0)];
                if cells
                    .iter()
                    .any(|&(cx, cy)| inside(fx, fy, cx, cy, cx + 17.0, cy + 17.0, 4.0))
                {
                    c = [255, 247, 235, 255];
                }
                rgba[i..i + 4].copy_from_slice(&c);
            }
        }
    }
    egui::IconData {
        rgba,
        width: n as u32,
        height: n as u32,
    }
}
