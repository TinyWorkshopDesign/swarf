//! Desktop UI (egui).

use crate::fonts::CjkFonts;
use eframe::egui::{self, Align, Color32, CornerRadius, Layout, RichText, Sense, Stroke, Ui, vec2};
use egui_plot::{Corner, Legend, Line, LineStyle, Plot, PlotPoints, Points, VLine};
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;
use swarf::calc::*;
use swarf::data::*;
use swarf::i18n::{K, Lang, material_name};

pub const APP_NAME: &str = "Swarf";

#[derive(Clone, Serialize, Deserialize)]
pub struct Saved {
    pub name: String,
    pub setup: Setup,
    #[serde(default)]
    pub compare: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum Tab {
    Results,
    Charts,
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct App {
    lang: Lang,
    setup: Setup,
    library: Vec<Saved>,
    new_name: String,
    tab: Tab,
    sweep_var: SweepVar,
    #[serde(skip)]
    pending_delete: Option<usize>,
    #[serde(skip)]
    copied_at: Option<f64>,
    #[serde(skip)]
    fonts: CjkFonts,
}

impl Default for App {
    fn default() -> Self {
        Self {
            lang: sys_locale::get_locale().map(|l| Lang::from_locale(&l)).unwrap_or(Lang::En),
            setup: Setup::default(),
            library: Vec::new(),
            new_name: String::new(),
            tab: Tab::Results,
            sweep_var: SweepVar::Radial,
            pending_delete: None,
            copied_at: None,
            fonts: CjkFonts::default(),
        }
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.all_styles_mut(|s| {
            s.spacing.item_spacing = vec2(8.0, 6.0);
            s.spacing.interact_size.y = 22.0;
        });
        let mut app: App = cc.storage.and_then(|st| eframe::get_value(st, eframe::APP_KEY)).unwrap_or_default();
        app.fonts.apply(&cc.egui_ctx, app.lang, app.lang.is_cjk());
        app
    }
}

/// App colours for light and dark themes.
struct Pal {
    lcd_bg: Color32,
    lcd_ink: Color32,
    lcd_dim: Color32,
    mrr: Color32,
    power: Color32,
    avail: Color32,
    ok: Color32,
    warn: Color32,
    crit: Color32,
    info: Color32,
    muted: Color32,
}

fn pal(dark: bool) -> Pal {
    if dark {
        Pal {
            lcd_bg: Color32::from_rgb(28, 40, 31),
            lcd_ink: Color32::from_rgb(185, 230, 166),
            lcd_dim: Color32::from_rgb(110, 140, 100),
            mrr: Color32::from_rgb(75, 176, 219),
            power: Color32::from_rgb(240, 160, 70),
            avail: Color32::from_rgb(150, 160, 170),
            ok: Color32::from_rgb(92, 196, 135),
            warn: Color32::from_rgb(229, 168, 74),
            crit: Color32::from_rgb(240, 118, 106),
            info: Color32::from_rgb(75, 176, 219),
            muted: Color32::from_rgb(147, 161, 173),
        }
    } else {
        Pal {
            lcd_bg: Color32::from_rgb(196, 207, 185),
            lcd_ink: Color32::from_rgb(27, 42, 30),
            lcd_dim: Color32::from_rgb(90, 108, 84),
            mrr: Color32::from_rgb(10, 106, 147),
            power: Color32::from_rgb(200, 110, 20),
            avail: Color32::from_rgb(110, 120, 130),
            ok: Color32::from_rgb(44, 122, 75),
            warn: Color32::from_rgb(168, 106, 18),
            crit: Color32::from_rgb(179, 54, 43),
            info: Color32::from_rgb(10, 106, 147),
            muted: Color32::from_rgb(86, 101, 115),
        }
    }
}

fn level_color(p: &Pal, l: Level) -> Color32 {
    match l {
        Level::Crit => p.crit,
        Level::Warn => p.warn,
        Level::Info => p.info,
    }
}

fn num(ui: &mut Ui, label: &str, v: &mut f64, range: RangeInclusive<f64>, speed: f64, suffix: &str, dec: usize) -> bool {
    ui.label(label);
    let r = ui.add(egui::DragValue::new(v).range(range).speed(speed).suffix(suffix).max_decimals(dec));
    ui.end_row();
    r.changed()
}

fn section(ui: &mut Ui, id: &str, title: &str, add: impl FnOnce(&mut Ui)) {
    // A fixed id keeps the open/closed state when the language changes.
    egui::CollapsingHeader::new(RichText::new(title.to_uppercase()).strong().size(15.0))
        .id_salt(id)
        .default_open(true)
        .show(ui, add);
    ui.add_space(4.0);
}

impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let p = pal(ui.visuals().dark_mode);
        let l = self.lang;

        egui::Panel::left("inputs")
            .resizable(true)
            .default_size(390.0)
            .size_range(330.0..=560.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(APP_NAME.to_uppercase()).size(28.0).strong());
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            self.language_picker(ui);
                        });
                    });
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(l.t(K::AppSubtitle)).color(p.muted));
                    });
                    if ui.small_button(l.t(K::LoadExample)).on_hover_text(l.t(K::LoadExampleTip)).clicked() {
                        self.setup = Setup::default();
                    }
                    ui.separator();
                    self.inputs(ui, &p);
                });
            });

        let o = compute(&self.setup);

        egui::CentralPanel::default().show(ui, |ui| {
            self.dro(ui, &o, &p);
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, Tab::Results, RichText::new(l.t(K::TabResults)).size(15.0));
                ui.selectable_value(&mut self.tab, Tab::Charts, RichText::new(l.t(K::TabCharts)).size(15.0));
            });
            ui.separator();
            egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| match self.tab {
                Tab::Results => self.results(ui, &o, &p),
                Tab::Charts => self.charts(ui, &o, &p),
            });
        });
    }
}

impl App {
    fn language_picker(&mut self, ui: &mut Ui) {
        let before = self.lang;
        let r = egui::ComboBox::from_id_salt("lang")
            .height(320.0)
            .selected_text(self.lang.native_name())
            .show_ui(ui, |ui| {
                for lang in Lang::ALL {
                    ui.selectable_value(&mut self.lang, lang, lang.native_name());
                }
            });
        let open = r.inner.is_some();
        let r = r.response.on_hover_text(self.lang.t(K::Language));
        // Load CJK fonts as soon as the menu may show their names, or a CJK language is chosen.
        self.fonts.apply(ui.ctx(), self.lang, r.hovered() || open || self.lang != before);
    }

    // ---------------------------------------------------------------- inputs
    fn inputs(&mut self, ui: &mut Ui, p: &Pal) {
        let l = self.lang;
        let s = &mut self.setup;

        section(ui, "machine", l.t(K::SecMachine), |ui| {
            let before = s.machine.preset;
            egui::ComboBox::from_id_salt("preset")
                .width(ui.available_width())
                .selected_text(l.preset(s.machine.preset))
                .show_ui(ui, |ui| {
                    for mp in MachinePreset::ALL {
                        ui.selectable_value(&mut s.machine.preset, mp, l.preset(mp));
                    }
                });
            if s.machine.preset != before && s.machine.preset != MachinePreset::Custom {
                s.machine = Machine::from_preset(s.machine.preset);
            }
            let m = &mut s.machine;
            let mut changed = false;
            egui::Grid::new("g-mach").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                changed |= num(ui, l.t(K::RpmMax), &mut m.rpm_max, 100.0..=100_000.0, 100.0, " rpm", 0);
                changed |= num(ui, l.t(K::RpmMin), &mut m.rpm_min, 0.0..=50_000.0, 50.0, " rpm", 0);
                changed |= num(ui, l.t(K::SpindlePower), &mut m.kw, 0.05..=100.0, 0.05, " kW", 2);
                ui.label(l.t(K::FullPowerFrom)).on_hover_text(l.t(K::FullPowerTip));
                changed |= ui.add(egui::DragValue::new(&mut m.rpm_base).range(1.0..=100_000.0).speed(100.0).suffix(" rpm").max_decimals(0)).changed();
                ui.end_row();
                changed |= num(ui, l.t(K::MaxFeed), &mut m.feed_max, 10.0..=100_000.0, 50.0, " mm/min", 0);
                ui.label(l.t(K::Rigidity));
                let r0 = m.rigidity;
                egui::ComboBox::from_id_salt("rigid").selected_text(l.rigidity(m.rigidity)).show_ui(ui, |ui| {
                    for r in Rigidity::ALL {
                        ui.selectable_value(&mut m.rigidity, r, l.rigidity(r));
                    }
                });
                changed |= r0 != m.rigidity;
                ui.end_row();
            });
            if changed {
                m.preset = MachinePreset::Custom;
            }
        });

        section(ui, "material", l.t(K::SecMaterial), |ui| {
            egui::ComboBox::from_id_salt("mat")
                .width(ui.available_width())
                .height(520.0)
                .selected_text(material_name(l, &s.material_id))
                .show_ui(ui, |ui| {
                    for g in Iso::ALL {
                        let [r, gg, b] = g.rgb();
                        ui.label(RichText::new(format!("{} · {}", g.letter(), l.iso(g))).strong().color(Color32::from_rgb(r, gg, b)));
                        for m in MATERIALS.iter().filter(|m| m.group == g) {
                            ui.selectable_value(&mut s.material_id, m.id.to_string(), format!("   {}", material_name(l, m.id)));
                        }
                    }
                });
            let m = s.material();
            ui.horizontal(|ui| {
                iso_badge(ui, m.group);
                ui.label(RichText::new(l.f(K::GroupLine, &[m.group.letter(), l.iso(m.group), &l.num(m.kc, 0)])).color(p.muted));
            });
        });

        section(ui, "tool", l.t(K::SecTool), |ui| {
            let t = &mut s.tool;
            let (kind0, d0) = (t.kind, t.d);
            egui::Grid::new("g-tool").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                ui.label(l.t(K::ToolType));
                egui::ComboBox::from_id_salt("kind").selected_text(l.tool_kind(t.kind)).show_ui(ui, |ui| {
                    for k in ToolKind::ALL {
                        ui.selectable_value(&mut t.kind, k, l.tool_kind(k));
                    }
                });
                ui.end_row();
                ui.label(l.t(K::ToolMat));
                egui::ComboBox::from_id_salt("tmat").selected_text(l.tool_material(t.material)).show_ui(ui, |ui| {
                    for k in ToolMaterial::ALL {
                        ui.selectable_value(&mut t.material, k, l.tool_material(k));
                    }
                });
                ui.end_row();
                ui.label(l.t(K::CoatingLabel));
                egui::ComboBox::from_id_salt("coat").selected_text(l.coating(t.coating)).show_ui(ui, |ui| {
                    for k in Coating::ALL {
                        ui.selectable_value(&mut t.coating, k, l.coating(k));
                    }
                });
                ui.end_row();
                num(ui, l.t(K::Diameter), &mut t.d, 0.1..=250.0, 0.1, " mm", 2);
                if t.kind != ToolKind::Drill {
                    ui.label(l.t(K::Flutes));
                    ui.add(egui::DragValue::new(&mut t.z).range(1..=40).speed(0.1));
                    ui.end_row();
                }
                match t.kind {
                    ToolKind::Bull => {
                        num(ui, l.t(K::CornerR), &mut t.corner_r, 0.0..=t.d / 2.0, 0.05, " mm", 2);
                    }
                    ToolKind::VBit => {
                        num(ui, l.t(K::VAngle), &mut t.v_angle, 10.0..=170.0, 1.0, "°", 0);
                        num(ui, l.t(K::TipD), &mut t.tip_d, 0.0..=t.d, 0.05, " mm", 2);
                    }
                    ToolKind::Face => {
                        num(ui, l.t(K::LeadAngle), &mut t.kappa, 10.0..=90.0, 1.0, "°", 0);
                    }
                    _ => {}
                }
                if t.kind != ToolKind::Face {
                    num(ui, l.t(K::FluteLen), &mut t.flute_len, 0.1..=400.0, 0.5, " mm", 1);
                    num(ui, l.t(K::Stickout), &mut t.stickout, 0.5..=500.0, 0.5, " mm", 1);
                    num(ui, l.t(K::ShankD), &mut t.shank_d, 0.5..=100.0, 0.1, " mm", 2);
                }
            });
            if t.d != d0 && t.d >= 3.0 && (t.shank_d - d0.ceil()).abs() < 1e-9 {
                t.shank_d = t.d.ceil(); // shank follows the diameter while they matched
            }
            if t.kind != kind0 {
                match t.kind {
                    ToolKind::Face => {
                        t.d = 50.0;
                        t.z = 5;
                        t.coating = Coating::TiCN;
                        t.kappa = 45.0;
                        s.apply_strategy(Strategy::Rough);
                    }
                    ToolKind::Drill => s.cut.ap = (t.d * 3.0 * 10.0).round() / 10.0,
                    ToolKind::VBit => s.cut.ap = 1.0,
                    _ => {}
                }
            }
        });

        let kind = s.tool.kind;
        section(ui, "cut", l.t(K::SecCut), |ui| {
            if kind != ToolKind::Drill {
                ui.horizontal_wrapped(|ui| {
                    for st in Strategy::ALL {
                        if ui.button(l.strategy(st)).clicked() {
                            s.apply_strategy(st);
                        }
                    }
                });
            }
            let c = &mut s.cut;
            egui::Grid::new("g-cut").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
                let lbl = if kind == ToolKind::Drill { l.t(K::HoleDepth) } else { l.t(K::AxialDepth) };
                num(ui, lbl, &mut c.ap, 0.01..=500.0, 0.05, " mm", 2);
                if !matches!(kind, ToolKind::Drill | ToolKind::VBit) {
                    num(ui, l.t(K::RadialWidth), &mut c.ae, 0.01..=500.0, 0.05, " mm", 2);
                }
            });
            if !matches!(kind, ToolKind::Drill | ToolKind::VBit) {
                ui.checkbox(&mut c.chip_thinning, l.t(K::ChipThinning));
            }
            if kind != ToolKind::Drill && kind != ToolKind::Face {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut c.limit_deflection, l.t(K::LimitDeflection));
                    let mut um = c.deflection_limit * 1000.0;
                    if ui.add(egui::DragValue::new(&mut um).range(1.0..=500.0).speed(1.0).suffix(" µm").max_decimals(0)).changed() {
                        c.deflection_limit = um / 1000.0;
                    }
                });
            }
            ui.checkbox(&mut c.limit_power, l.t(K::LimitPower));
        });

        section(ui, "tweaks", l.t(K::SecTweaks), |ui| {
            let tw = &mut s.tweaks;
            let v = tw.vc_factor();
            slider(ui, p, l.t(K::ToolLife), l.t(K::Productivity), &mut tw.life, format!("Vc ×{}", l.num(v, 2)));
            ui.add_space(4.0);
            let f = tw.fz_factor();
            slider(ui, p, l.t(K::FinishEnd), l.t(K::RemovalEnd), &mut tw.finish, format!("fz ×{}", l.num(f, 2)));
        });

        section(ui, "library", l.t(K::SecLibrary), |ui| {
            ui.horizontal(|ui| {
                let w = ui.available_width() - 80.0;
                ui.add(egui::TextEdit::singleline(&mut self.new_name).hint_text(l.t(K::NameHint)).desired_width(w));
                if ui.button(l.t(K::Save)).clicked() {
                    let name = if self.new_name.trim().is_empty() {
                        l.f(K::DefaultName, &[&(self.library.len() + 1).to_string()])
                    } else {
                        self.new_name.trim().to_string()
                    };
                    self.library.insert(0, Saved { name, setup: self.setup.clone(), compare: true });
                    self.new_name.clear();
                }
            });
            if self.library.is_empty() {
                ui.label(RichText::new(l.t(K::LibEmpty)).color(p.muted));
            }
            let mut load = None;
            let mut delete = None;
            for (i, item) in self.library.iter_mut().enumerate() {
                egui::Frame::group(ui.style()).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&item.name).strong());
                            let sub = format!("Ø{} Z{} · {}", l.num(item.setup.tool.d, 1), item.setup.tool.z, material_name(l, &item.setup.material_id));
                            ui.label(RichText::new(sub).small().color(p.muted));
                        });
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let del = if self.pending_delete == Some(i) { l.t(K::Confirm) } else { l.t(K::Delete) };
                            if ui.small_button(del).clicked() {
                                if self.pending_delete == Some(i) {
                                    delete = Some(i);
                                } else {
                                    self.pending_delete = Some(i);
                                }
                            }
                            if ui.small_button(l.t(K::Load)).clicked() {
                                load = Some(i);
                            }
                            ui.checkbox(&mut item.compare, "").on_hover_text(l.t(K::CompareTip));
                        });
                    });
                });
            }
            if let Some(i) = load {
                self.setup = self.library[i].setup.clone();
                self.pending_delete = None;
            }
            if let Some(i) = delete {
                self.library.remove(i);
                self.pending_delete = None;
            }
        });
    }

    // ---------------------------------------------------------------- DRO
    fn dro(&mut self, ui: &mut Ui, o: &Outcome, p: &Pal) {
        let l = self.lang;
        egui::Frame::new()
            .fill(p.lcd_bg)
            .stroke(Stroke::new(3.0, ui.visuals().strong_text_color()))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(egui::Margin::symmetric(18, 12))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    dro_axis(ui, p, "S", &l.num(o.rpm, 0), "rpm");
                    ui.add_space(40.0);
                    dro_axis(ui, p, "F", &l.num(o.feed.round(), 0), "mm/min");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        let copied = self.copied_at.is_some_and(|t| ui.input(|i| i.time) - t < 1.5);
                        if ui.button(if copied { l.t(K::Copied) } else { l.t(K::CopyGcode) }).clicked() {
                            ui.ctx().copy_text(o.gcode());
                            self.copied_at = Some(ui.input(|i| i.time));
                        }
                        ui.label(RichText::new(o.gcode()).monospace().size(14.0).color(p.lcd_ink));
                    });
                });
                ui.add(egui::Separator::default().spacing(6.0));
                ui.horizontal_wrapped(|ui| {
                    let sub = if o.kind == ToolKind::Drill {
                        l.f(K::SubDrill, &[&l.num(o.vc, 0), &l.num(o.fz, 3), l.t(K::UnitMmRev)])
                    } else {
                        l.f(K::SubMill, &[&l.num(o.vc, 0), &l.num(o.fz, 4), &l.num(o.chip, 4)])
                    };
                    ui.label(RichText::new(sub).monospace().color(p.lcd_dim));
                    ui.add_space(12.0);
                    if o.limiters.is_empty() {
                        chip(ui, l.t(K::NoLimit), p.ok);
                    }
                    for lim in &o.limiters {
                        chip(ui, &l.f(K::LimitChip, &[l.limiter(*lim)]), p.warn);
                    }
                });
            });
    }

    // ---------------------------------------------------------------- results
    fn results(&mut self, ui: &mut Ui, o: &Outcome, p: &Pal) {
        let l = self.lang;
        let material_id = self.setup.material_id.clone();
        let cols = if ui.available_width() > 760.0 { 2 } else { 1 };
        ui.columns(cols, |c| {
            let (left, right) = if cols == 2 {
                let (a, b) = c.split_at_mut(1);
                (&mut a[0], Some(&mut b[0]))
            } else {
                (&mut c[0], None)
            };
            stats(left, l, o, &self.setup, p);
            let ui = match right {
                Some(r) => r,
                None => left,
            };
            ui.label(RichText::new(l.t(K::Warnings).to_uppercase()).strong());
            if o.warnings.is_empty() {
                alert(ui, p.ok, l.t(K::BalancedTitle), l.t(K::BalancedText));
            }
            for w in &o.warnings {
                let (title, text) = l.warning(w, &material_id);
                alert(ui, level_color(p, w.level), &title, &text);
            }
            ui.add_space(8.0);
            egui::CollapsingHeader::new(RichText::new(l.t(K::WhyNumbers).to_uppercase()).strong())
                .id_salt("why")
                .default_open(true)
                .show(ui, |ui| {
                    egui::Grid::new("why").num_columns(2).striped(true).spacing([16.0, 4.0]).show(ui, |ui| {
                        for r in &o.why {
                            let (label, value) = l.row(r);
                            if r.total {
                                ui.label(RichText::new(label).strong());
                                ui.label(RichText::new(value).monospace().strong());
                            } else {
                                ui.label(label);
                                ui.label(RichText::new(value).monospace());
                            }
                            ui.end_row();
                        }
                    });
                });
        });
        ui.add_space(10.0);
        ui.label(RichText::new(l.t(K::Disclaimer)).small().color(p.muted));
    }

    // ---------------------------------------------------------------- charts
    fn charts(&mut self, ui: &mut Ui, o: &Outcome, p: &Pal) {
        let l = self.lang;
        let kind = self.setup.tool.kind;
        let radial_ok = !matches!(kind, ToolKind::Drill | ToolKind::VBit);
        if !radial_ok {
            self.sweep_var = SweepVar::Axial;
        }
        let depth_label = if kind == ToolKind::Drill { l.t(K::HoleDepth) } else { l.t(K::AxialShort) };
        ui.horizontal(|ui| {
            ui.label(RichText::new(l.t(K::Curve).to_uppercase()).strong());
            ui.label(l.t(K::Vary));
            ui.add_enabled_ui(radial_ok, |ui| {
                ui.selectable_value(&mut self.sweep_var, SweepVar::Radial, l.t(K::RadialWidth));
            });
            ui.selectable_value(&mut self.sweep_var, SweepVar::Axial, depth_label);
        });
        ui.label(RichText::new(l.t(K::CurveNote)).small().color(p.muted));

        let var = self.sweep_var;
        let pts = sweep(&self.setup, var, 80);
        let (cur_x, xlabel) = match var {
            SweepVar::Radial => (self.setup.cut.ae / self.setup.tool.d.max(0.1) * 100.0, l.t(K::XRadial).to_string()),
            SweepVar::Axial => (self.setup.cut.ap, format!("{depth_label} (mm)")),
        };
        let h = ((ui.available_height() - 360.0) / 2.0).clamp(190.0, 300.0);

        let mrr: Vec<[f64; 2]> = pts.iter().map(|q| [q.x, q.mrr]).collect();
        let mrr_lim: Vec<[f64; 2]> = pts.iter().filter(|q| q.limited).map(|q| [q.x, q.mrr]).collect();
        let pw: Vec<[f64; 2]> = pts.iter().map(|q| [q.x, q.spindle_kw]).collect();
        let pw_lim: Vec<[f64; 2]> = pts.iter().filter(|q| q.limited).map(|q| [q.x, q.spindle_kw]).collect();
        let av: Vec<[f64; 2]> = pts.iter().map(|q| [q.x, q.avail_kw]).collect();
        let (cur_mrr, cur_kw) = (o.mrr, o.spindle_kw);
        let legend = || Legend::default().position(Corner::RightBottom);

        ui.label(RichText::new(l.t(K::MrrTitle)).strong().color(p.mrr));
        Plot::new("mrr")
            .height(h)
            .link_axis("sweep", [true, false])
            .link_cursor("sweep", [true, false])
            .allow_scroll(false)
            .allow_zoom(false)
            .allow_drag(false)
            .include_y(0.0)
            .y_axis_label("cm³/min")
            .legend(legend())
            .show(ui, |pu| {
                pu.line(Line::new("MRR", PlotPoints::from(mrr)).color(p.mrr).width(2.0));
                if !mrr_lim.is_empty() {
                    pu.points(Points::new(l.t(K::LegLimited), PlotPoints::from(mrr_lim)).color(p.warn).radius(2.0));
                }
                pu.vline(VLine::new("", cur_x).color(p.muted));
                pu.points(Points::new(l.t(K::LegCurrent), vec![[cur_x, cur_mrr]]).color(p.mrr).radius(6.0).filled(true));
            });
        ui.add_space(6.0);
        ui.label(RichText::new(l.t(K::PowerTitle)).strong().color(p.power));
        Plot::new("power")
            .height(h)
            .link_axis("sweep", [true, false])
            .link_cursor("sweep", [true, false])
            .allow_scroll(false)
            .allow_zoom(false)
            .allow_drag(false)
            .include_y(0.0)
            .x_axis_label(xlabel)
            .y_axis_label("kW")
            .legend(legend())
            .show(ui, |pu| {
                pu.line(Line::new(l.t(K::LegAvailable), PlotPoints::from(av)).color(p.avail).width(1.5).style(LineStyle::dashed_loose()));
                pu.line(Line::new(l.t(K::LegRequired), PlotPoints::from(pw)).color(p.power).width(2.0));
                if !pw_lim.is_empty() {
                    pu.points(Points::new(l.t(K::LegLimited), PlotPoints::from(pw_lim)).color(p.warn).radius(2.0));
                }
                pu.vline(VLine::new("", cur_x).color(p.muted));
                pu.points(Points::new(l.t(K::LegCurrent), vec![[cur_x, cur_kw]]).color(p.power).radius(6.0).filled(true));
            });

        ui.add_space(14.0);
        ui.separator();
        ui.label(RichText::new(l.t(K::Compare).to_uppercase()).strong());
        ui.label(RichText::new(l.t(K::CompareNote)).small().color(p.muted));
        ui.add_space(4.0);

        let mut rows: Vec<(String, Outcome, bool)> = vec![(l.t(K::CurrentCut).into(), o.clone(), true)];
        if kind != ToolKind::Drill {
            for st in Strategy::ALL {
                let mut s = self.setup.clone();
                s.apply_strategy(st);
                rows.push((l.strategy(st).into(), compute(&s), false));
            }
        }
        for sv in self.library.iter().filter(|s| s.compare) {
            rows.push((sv.name.clone(), compute(&sv.setup), false));
        }
        compare_bars(ui, l, &rows, p);
    }
}

// ---------------------------------------------------------------- widgets

fn iso_badge(ui: &mut Ui, g: Iso) {
    let [r, gg, b] = g.rgb();
    let (rect, _) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::hover());
    ui.painter().rect_filled(rect, 3.0, Color32::from_rgb(r, gg, b));
    let txt = if g == Iso::M { Color32::from_rgb(27, 27, 27) } else { Color32::WHITE };
    ui.painter().text(rect.center(), egui::Align2::CENTER_CENTER, g.letter(), egui::FontId::proportional(15.0), txt);
}

fn dro_axis(ui: &mut Ui, p: &Pal, axis: &str, value: &str, unit: &str) {
    ui.label(RichText::new(axis).size(30.0).strong().color(p.lcd_ink));
    ui.label(RichText::new(value).monospace().size(46.0).strong().color(p.lcd_ink));
    ui.label(RichText::new(unit).monospace().size(12.0).color(p.lcd_dim));
}

fn chip(ui: &mut Ui, text: &str, color: Color32) {
    egui::Frame::new()
        .stroke(Stroke::new(1.0, color))
        .corner_radius(CornerRadius::same(3))
        .inner_margin(egui::Margin::symmetric(6, 1))
        .show(ui, |ui| {
            ui.label(RichText::new(text).monospace().size(11.0).color(color));
        });
}

fn slider(ui: &mut Ui, p: &Pal, left: &str, right: &str, v: &mut f64, value: String) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(left.to_uppercase()).small().strong().color(p.muted));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(right.to_uppercase()).small().strong().color(p.muted));
        });
    });
    ui.spacing_mut().slider_width = ui.available_width() - 70.0;
    ui.add(egui::Slider::new(v, -1.0..=1.0).step_by(0.05).show_value(false));
    ui.label(RichText::new(value).monospace().small());
}

fn alert(ui: &mut Ui, color: Color32, title: &str, text: &str) {
    egui::Frame::new()
        .fill(color.gamma_multiply(0.10))
        .stroke(Stroke::new(1.0, color.gamma_multiply(0.5)))
        .corner_radius(CornerRadius::same(4))
        .inner_margin(egui::Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(RichText::new(title).strong().color(color));
            ui.label(text);
        });
    ui.add_space(2.0);
}

/// Usage gauge: green, amber above 85 %, red when over the maximum.
fn gauge(ui: &mut Ui, value: f64, max: f64, p: &Pal) {
    let frac = if max > 0.0 { (value / max).clamp(0.0, 1.0) } else { 1.0 };
    let color = if value > max * 1.001 {
        p.crit
    } else if frac > 0.85 {
        p.warn
    } else {
        p.ok
    };
    bar(ui, frac, color);
}

fn bar(ui: &mut Ui, frac: f64, color: Color32) {
    ui.add(egui::ProgressBar::new(frac as f32).fill(color).desired_height(6.0).desired_width(ui.available_width()));
}

fn stat(ui: &mut Ui, p: &Pal, key: &str, value: String, unit: &str, sub: Option<String>) {
    ui.label(RichText::new(key.to_uppercase()).small().strong().color(p.muted));
    ui.horizontal(|ui| {
        ui.label(RichText::new(value).monospace().size(19.0));
        if !unit.is_empty() {
            ui.label(RichText::new(unit).small().color(p.muted));
        }
    });
    if let Some(s) = sub {
        ui.label(RichText::new(s).small().color(p.muted));
    }
}

fn stats(ui: &mut Ui, l: Lang, o: &Outcome, s: &Setup, p: &Pal) {
    ui.label(RichText::new(l.t(K::Details).to_uppercase()).strong());
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.set_width(ui.available_width());
        if let Some(dr) = &o.drill {
            stat(ui, p, l.t(K::FeedPerRev), l.num(o.fz, 3), l.t(K::UnitMmRev), Some(l.f(K::TargetX, &[&l.num(o.chip_target, 3)])));
            ui.separator();
            stat(ui, p, l.t(K::Mrr), l.num(o.mrr, 1), "cm³/min", None);
            ui.separator();
            stat(ui, p, l.t(K::SpindlePower), l.num(o.spindle_kw, 2), "kW", Some(l.f(K::AvailKw, &[&l.num(o.avail_kw, 2)])));
            gauge(ui, o.spindle_kw, o.avail_kw, p);
            ui.separator();
            stat(ui, p, l.t(K::Torque), l.num(o.torque, 2), "N·m", None);
            ui.separator();
            stat(ui, p, l.t(K::Thrust), l.num(dr.thrust_n, 0), "N", Some(format!("kc {} N/mm²", l.num(o.kc, 0))));
            ui.separator();
            stat(ui, p, l.t(K::HoleTime), l.num(dr.time_s, 1), "s", Some(l.f(K::FeedOnly, &[&l.num(dr.depth, 1)])));
            return;
        }
        let thin = if o.thinning > 1.001 { l.f(K::ThinningX, &[&l.num(o.thinning, 2)]) } else { String::new() };
        stat(ui, p, l.t(K::ActualChip), l.num(o.chip, 4), "mm", Some(format!("{}{thin}", l.f(K::ProgFz, &[&l.num(o.fz, 4)]))));
        // Full is good here: the bar shows how much of the target chip survives the limits.
        let frac = (o.chip / o.chip_target.max(1e-9)).clamp(0.0, 1.0);
        bar(ui, frac, if frac < 0.6 { p.warn } else { p.ok });
        ui.separator();
        stat(ui, p, l.t(K::Mrr), l.num(o.mrr, 1), "cm³/min", Some(format!("ap {} × ae {} mm", l.num(o.ap, 2), l.num(o.ae, 2))));
        ui.separator();
        stat(ui, p, l.t(K::SpindlePower), l.num(o.spindle_kw, 2), "kW", Some(l.f(K::AvailKwAt, &[&l.num(o.avail_kw, 2), &l.num(o.rpm, 0)])));
        gauge(ui, o.spindle_kw, o.avail_kw, p);
        ui.separator();
        stat(ui, p, l.t(K::Torque), l.num(o.torque, 2), "N·m", Some(l.f(K::ForceKc, &[&l.num(o.force, 0), &l.num(o.kc, 0)])));
        ui.separator();
        if let Some(d) = o.deflection {
            stat(ui, p, l.t(K::TipDeflection), l.num(d * 1000.0, 1), "µm", Some(l.f(K::DeflSub, &[&l.num(o.deflection_limit * 1000.0, 0), &l.num(s.tool.stickout, 0)])));
            gauge(ui, d, o.deflection_limit, p);
            ui.separator();
            let plunge = o.plunge_feed().min(s.machine.feed_max);
            stat(
                ui,
                p,
                l.t(K::RampPlunge),
                format!("{}°", l.num(o.group.ramp_deg(), 1)),
                "",
                Some(l.f(K::RampSub, &[&l.num(plunge, 0), &l.num(o.d * 1.3, 1), &l.num(o.d * 1.8, 1)])),
            );
        } else {
            stat(ui, p, l.t(K::Entry), l.t(K::EntryValue).into(), "", Some(l.t(K::EntrySub).into()));
        }
        match o.kind {
            ToolKind::Ball | ToolKind::Bull => {
                let r = if o.kind == ToolKind::Ball { o.d / 2.0 } else { s.tool.corner_r.max(0.001) };
                let h = r - (r * r - (o.ae / 2.0).powi(2)).max(0.0).sqrt();
                ui.separator();
                stat(ui, p, l.t(K::Scallop), l.num(h * 1000.0, 1), "µm", Some(l.f(K::ScallopSub, &[&l.num(o.ae, 2)])));
            }
            ToolKind::Flat => {
                let h = o.fz * o.fz / (4.0 * o.d);
                ui.separator();
                stat(ui, p, l.t(K::FeedCusp), l.num(h * 1000.0, 2), "µm", Some(l.f(K::RaSub, &[&l.num(h * 1000.0 / 4.0, 2)])));
            }
            _ => {}
        }
    });
}

/// Side-by-side horizontal bars: MRR and power (with a tick for available power).
fn compare_bars(ui: &mut Ui, l: Lang, rows: &[(String, Outcome, bool)], p: &Pal) {
    let max_mrr = rows.iter().map(|r| r.1.mrr).fold(0.0, f64::max).max(1e-6);
    let max_kw = rows.iter().map(|r| r.1.spindle_kw.max(r.1.avail_kw)).fold(0.0, f64::max).max(1e-6);
    let name_w = 170.0f32;
    let bar_w = ((ui.available_width() - name_w - 24.0) / 2.0).max(120.0);

    ui.horizontal(|ui| {
        ui.add_sized([name_w, 16.0], egui::Label::new(""));
        ui.add_sized([bar_w, 16.0], egui::Label::new(RichText::new("MRR cm³/min").small().strong().color(p.mrr)));
        ui.add_sized([bar_w, 16.0], egui::Label::new(RichText::new(l.t(K::BarPower)).small().strong().color(p.power)));
    });
    for (name, o, current) in rows {
        let limits = if o.limiters.is_empty() {
            l.t(K::NoLimit).to_string()
        } else {
            o.limiters.iter().map(|x| l.limiter(*x)).collect::<Vec<_>>().join(", ")
        };
        ui.horizontal(|ui| {
            let txt = RichText::new(name);
            ui.add_sized([name_w, 26.0], egui::Label::new(if *current { txt.strong() } else { txt }).truncate());
            hbar(ui, bar_w, o.mrr / max_mrr, p.mrr, &l.num(o.mrr, 1), None, p);
            let col = if o.power_ok() { p.power } else { p.crit };
            hbar(ui, bar_w, o.spindle_kw / max_kw, col, &l.num(o.spindle_kw, 2), Some((o.avail_kw / max_kw) as f32), p);
        })
        .response
        .on_hover_text(format!("S {} · F {} mm/min\nap {} · ae {} mm\n{limits}", l.num(o.rpm, 0), l.num(o.feed, 0), l.num(o.ap, 2), l.num(o.ae, 2)));
    }
}

fn hbar(ui: &mut Ui, width: f32, frac: f64, color: Color32, label: &str, marker: Option<f32>, p: &Pal) {
    let (rect, _) = ui.allocate_exact_size(vec2(width, 22.0), Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 2.0, ui.visuals().faint_bg_color);
    let usable = (rect.width() - 58.0).max(10.0); // room for the value label
    let mut fill = rect;
    fill.set_width((usable * frac.clamp(0.0, 1.0) as f32).max(1.0));
    painter.rect_filled(fill, 2.0, color);
    if let Some(m) = marker {
        let x = rect.left() + usable * m.clamp(0.0, 1.0);
        painter.line_segment([egui::pos2(x, rect.top() - 1.0), egui::pos2(x, rect.bottom() + 1.0)], Stroke::new(2.0, p.avail));
    }
    painter.text(egui::pos2(fill.right() + 6.0, rect.center().y), egui::Align2::LEFT_CENTER, label, egui::FontId::monospace(12.0), ui.visuals().text_color());
}
