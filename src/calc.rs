//! Speeds and feeds engine. Pure: no UI, no language; text is produced by `i18n`.
//!
//! Models:
//! - radial chip thinning (ae < Ø/2) and lead-angle / ball-nose thinning;
//! - mean chip thickness and Kienzle specific force (kc = kc1.1 · hm^-mc);
//! - milling power Pc = ap·ae·vf·kc / 6·10⁷ (kW), drilling Pc = f·vc·Ø·kc / 240 000;
//! - tool deflection as a two-segment cantilever (shank + flutes at 0.8·Ø).

use crate::data::*;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Spindle mechanical efficiency: cutting power = spindle power · η.
pub const SPINDLE_EFF: f64 = 0.85;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Machine {
    pub preset: MachinePreset,
    pub rpm_max: f64,
    pub rpm_min: f64,
    pub kw: f64,
    /// Below this speed power drops proportionally (constant torque region).
    pub rpm_base: f64,
    pub feed_max: f64,
    pub rigidity: Rigidity,
}

impl Machine {
    pub fn from_preset(p: MachinePreset) -> Self {
        match p {
            MachinePreset::Router | MachinePreset::Custom => Self {
                preset: p,
                rpm_max: 24000.0,
                rpm_min: 8000.0,
                kw: 2.2,
                rpm_base: 24000.0,
                feed_max: 5000.0,
                rigidity: Rigidity::Low,
            },
            MachinePreset::Bench => Self {
                preset: p,
                rpm_max: 10000.0,
                rpm_min: 100.0,
                kw: 1.1,
                rpm_base: 3000.0,
                feed_max: 2500.0,
                rigidity: Rigidity::Medium,
            },
            MachinePreset::Vmc => Self {
                preset: p,
                rpm_max: 12000.0,
                rpm_min: 50.0,
                kw: 15.0,
                rpm_base: 1500.0,
                feed_max: 30000.0,
                rigidity: Rigidity::High,
            },
        }
    }
}

impl Default for Machine {
    fn default() -> Self {
        Self::from_preset(MachinePreset::Router)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Tool {
    pub kind: ToolKind,
    pub material: ToolMaterial,
    pub coating: Coating,
    pub d: f64,
    pub z: u32,
    /// Corner radius of a bull-nose end mill.
    pub corner_r: f64,
    /// Included angle of a V-bit.
    pub v_angle: f64,
    /// Tip diameter of a V-bit.
    pub tip_d: f64,
    /// Lead angle of a face mill.
    pub kappa: f64,
    pub flute_len: f64,
    pub stickout: f64,
    pub shank_d: f64,
}

impl Default for Tool {
    fn default() -> Self {
        Self {
            kind: ToolKind::Flat,
            material: ToolMaterial::Carbide,
            coating: Coating::ZrN,
            d: 6.0,
            z: 3,
            corner_r: 1.0,
            v_angle: 90.0,
            tip_d: 0.2,
            kappa: 45.0,
            flute_len: 18.0,
            stickout: 25.0,
            shank_d: 6.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Cut {
    pub ap: f64,
    pub ae: f64,
    pub chip_thinning: bool,
    pub limit_deflection: bool,
    pub deflection_limit: f64,
    pub limit_power: bool,
}

impl Default for Cut {
    fn default() -> Self {
        Self {
            ap: 6.0,
            ae: 1.2,
            chip_thinning: true,
            limit_deflection: true,
            deflection_limit: 0.025,
            limit_power: true,
        }
    }
}

/// User adjustments, both from -1 to +1.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Tweaks {
    /// -1 = longer tool life, +1 = more productivity.
    pub life: f64,
    /// -1 = better finish, +1 = more material removal.
    pub finish: f64,
}

impl Tweaks {
    pub fn vc_factor(&self) -> f64 {
        1.0 + self.life.clamp(-1.0, 1.0) * 0.25
    }
    pub fn fz_factor(&self) -> f64 {
        let f = self.finish.clamp(-1.0, 1.0);
        if f < 0.0 { 1.0 + f * 0.4 } else { 1.0 + f * 0.3 }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Setup {
    pub machine: Machine,
    pub material_id: String,
    pub tool: Tool,
    pub cut: Cut,
    pub tweaks: Tweaks,
}

impl Default for Setup {
    fn default() -> Self {
        Self {
            machine: Machine::default(),
            material_id: "al6061".into(),
            tool: Tool::default(),
            cut: Cut::default(),
            tweaks: Tweaks::default(),
        }
    }
}

impl Setup {
    pub fn material(&self) -> &'static Material {
        material(&self.material_id)
    }

    /// Sets ap and ae for a strategy, based on material and tool.
    pub fn apply_strategy(&mut self, st: Strategy) {
        let g = self.material().group;
        let d = self.tool.d;
        let (soft, hard) = (g.is_soft(), g.is_hard());
        let (mut ap, mut ae) = match st {
            Strategy::Slot => (
                d * if soft {
                    0.75
                } else if matches!(g, Iso::P | Iso::K) {
                    0.4
                } else {
                    0.2
                },
                d,
            ),
            Strategy::Rough => (
                d * if soft {
                    1.5
                } else if hard {
                    0.8
                } else {
                    1.0
                },
                d * if soft {
                    0.4
                } else if hard {
                    0.2
                } else {
                    0.3
                },
            ),
            Strategy::Hsm => (
                d * 2.5,
                d * if soft {
                    0.15
                } else if hard {
                    0.07
                } else {
                    0.1
                },
            ),
            Strategy::Finish => (d * 1.5, (d * 0.03).max(0.1)),
        };
        match self.tool.kind {
            ToolKind::Face => {
                ap = match st {
                    Strategy::Finish => 0.3,
                    Strategy::Rough if soft => 3.0,
                    Strategy::Rough => 2.0,
                    _ => 1.5,
                };
                ae = d * if st == Strategy::Slot { 1.0 } else { 0.7 };
            }
            ToolKind::Ball if st == Strategy::Finish => {
                ae = d * 0.08;
                ap = d * 0.1;
            }
            _ => {}
        }
        if self.tool.kind != ToolKind::Face {
            ap = ap.min(self.tool.flute_len);
        }
        self.cut.ap = round2(ap);
        self.cut.ae = round2(ae);
    }
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Crit,
    Warn,
    Info,
}

/// A warning, with the numbers it needs. `i18n` turns it into text.
#[derive(Clone, Debug, PartialEq)]
pub enum Msg {
    HssNotSuitable,
    AlTiNOnAluminium,
    DiamondOnFerrous,
    PowerShort { need_kw: f64, avail_kw: f64, rpm: f64 },
    DeepHole { ratio: f64, peck: f64 },
    Over8D,
    VBitTooDeep { ap: f64 },
    ThinChip { chip: f64, metal: bool },
    DeflectionHalvesFeed,
    PowerHalvesFeed,
    Deflection { um: f64, limit_um: f64 },
    ApOverFlute { flute: f64 },
    LongStickout { ratio: f64 },
    StickoutShorterThanFlute,
    TooManyFlutesInAlSlot { z: u32 },
    DeepSlot,
    RpmCapped { vc: f64, pct: f64 },
    RpmMinTooHigh { vc: f64, target: f64 },
}

#[derive(Clone, Debug)]
pub struct Warning {
    pub level: Level,
    pub msg: Msg,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Limiter {
    RpmMax,
    RpmMin,
    FeedMax,
    Power,
    Deflection,
}

/// One step of the "where the numbers come from" breakdown.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Step {
    VcBase(ToolMaterial),
    Drilling,
    Coating(Coating),
    FullSlot,
    Engagement { pct: f64, cooler: bool },
    LifeTweak,
    VcTarget,
    EffectiveDiameter,
    FzBase { cf: f64 },
    FzBaseInsert,
    FzFactors { tool: f64, rigidity: f64, tweak: f64, slot: bool },
    ChipTarget,
    RadialThinning,
    BallThinning,
    LeadThinning { kappa: f64 },
    ThinningCapped,
    Reduction(Limiter),
    FzProgrammed,
    FnBase { cf: f64 },
    FnFactors,
    FnTarget,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    MPerMin,
    Mm,
    MmPerRev,
    Factor,
    None,
}

#[derive(Clone, Debug)]
pub struct Row {
    pub step: Step,
    pub value: f64,
    pub unit: Unit,
    pub decimals: usize,
    pub total: bool,
}

#[derive(Clone, Debug, Default)]
pub struct DrillInfo {
    pub thrust_n: f64,
    pub time_s: f64,
    pub depth: f64,
    pub peck: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct Outcome {
    pub kind: ToolKind,
    pub group: Iso,
    pub rpm: f64,
    pub vc: f64,
    pub vc_target: f64,
    pub feed: f64,
    /// Programmed feed per tooth (milling) or feed per revolution (drilling).
    pub fz: f64,
    /// Actual maximum chip thickness.
    pub chip: f64,
    pub chip_target: f64,
    pub thinning: f64,
    pub mrr: f64,
    /// Power required at the spindle (kW).
    pub spindle_kw: f64,
    /// Power available at the spindle at this speed (kW).
    pub avail_kw: f64,
    pub torque: f64,
    pub force: f64,
    pub kc: f64,
    pub deflection: Option<f64>,
    pub deflection_limit: f64,
    pub d: f64,
    pub d_eff: f64,
    pub ap: f64,
    pub ae: f64,
    pub radial_ratio: f64,
    pub kappa: f64,
    pub z: u32,
    pub limiters: Vec<Limiter>,
    pub warnings: Vec<Warning>,
    pub why: Vec<Row>,
    pub drill: Option<DrillInfo>,
}

impl Outcome {
    pub fn power_ok(&self) -> bool {
        self.spindle_kw <= self.avail_kw * 1.001
    }
    pub fn gcode(&self) -> String {
        let s = format!("S{} M3\n", self.rpm.round() as i64);
        match &self.drill {
            Some(dr) => {
                let depth = trim_num(dr.depth, 3);
                match dr.peck {
                    Some(q) => format!("{s}G83 Z-{depth} R1. Q{} F{}", trim_num(q, 1), self.feed.round() as i64),
                    None => format!("{s}G81 Z-{depth} R1. F{}", self.feed.round() as i64),
                }
            }
            None => format!("{s}G1 F{}", self.feed.round() as i64),
        }
    }
    pub fn plunge_feed(&self) -> f64 {
        self.chip * 0.5 * self.z as f64 * self.rpm
    }
}

/// G-code number: dot decimal, trailing zeros removed, always a decimal point.
fn trim_num(v: f64, dec: usize) -> String {
    let s = format!("{v:.dec$}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s.contains('.') { s.to_string() } else { format!("{s}.") }
}

fn dterm(d: f64) -> f64 {
    if d <= 12.0 { d } else { 12.0 * (d / 12.0).sqrt() }
}

/// Tip deflection of a two-segment cantilever: shank Ø ds and flutes as 0.8·Ø.
pub fn deflection(force: f64, stickout: f64, flute_len: f64, d: f64, ds: f64, e: f64) -> f64 {
    let i1 = PI * ds.powi(4) / 64.0;
    let i2 = PI * (0.8 * d).powi(4) / 64.0;
    let b = flute_len.min(stickout);
    force / (3.0 * e) * ((stickout.powi(3) - b.powi(3)) / i1 + b.powi(3) / i2)
}

struct Log {
    why: Vec<Row>,
    warnings: Vec<Warning>,
}

impl Log {
    fn row(&mut self, step: Step, value: f64, unit: Unit, decimals: usize) {
        self.why.push(Row { step, value, unit, decimals, total: false });
    }
    fn total(&mut self, step: Step, value: f64, unit: Unit, decimals: usize) {
        self.why.push(Row { step, value, unit, decimals, total: true });
    }
    fn factor(&mut self, step: Step, value: f64) {
        self.row(step, value, Unit::Factor, 2);
    }
    fn warn(&mut self, level: Level, msg: Msg) {
        self.warnings.push(Warning { level, msg });
    }
    fn sorted_warnings(mut self) -> (Vec<Row>, Vec<Warning>) {
        self.warnings.sort_by_key(|w| w.level);
        (self.why, self.warnings)
    }
}

pub fn compute(s: &Setup) -> Outcome {
    let m = s.material();
    let g = m.group;
    let metal = g.is_metal();
    let tool = &s.tool;
    let mach = &s.machine;
    let mut log = Log { why: Vec::new(), warnings: Vec::new() };

    let d = tool.d.max(0.1);
    let life_m = s.tweaks.vc_factor();
    let fin_m = s.tweaks.fz_factor();
    let tm_f = if tool.material == ToolMaterial::Carbide { 1.0 } else { 0.8 };
    let rigid_f = if metal { mach.rigidity.factor() } else { 1.0 };

    let mut vc_base = match tool.material {
        ToolMaterial::Carbide => m.vc_carbide,
        ToolMaterial::HssCo => m.vc_hss * 1.15,
        ToolMaterial::Hss => m.vc_hss,
    };
    if tool.material != ToolMaterial::Carbide && m.vc_hss == 0.0 {
        log.warn(Level::Crit, Msg::HssNotSuitable);
        vc_base = m.vc_carbide * 0.2;
    }
    let coat_m = tool.coating.vc_factor(g);
    if tool.coating == Coating::AlTiN && g == Iso::N {
        log.warn(Level::Warn, Msg::AlTiNOnAluminium);
    }
    if tool.coating == Coating::Diamond && matches!(g, Iso::P | Iso::M | Iso::K | Iso::H) {
        log.warn(Level::Crit, Msg::DiamondOnFerrous);
    }

    let rpm_max = mach.rpm_max.max(1.0);
    let rpm_min = mach.rpm_min.clamp(0.0, rpm_max);
    let feed_max = mach.feed_max.max(1.0);
    let mut limiters = Vec::new();
    let clamp_rpm = |n: f64, lim: &mut Vec<Limiter>| -> f64 {
        let mut n = n;
        if n > rpm_max {
            n = rpm_max;
            lim.push(Limiter::RpmMax);
        }
        if n < rpm_min.max(1.0) {
            n = rpm_min.max(1.0);
            lim.push(Limiter::RpmMin);
        }
        n.round()
    };
    let avail = |n: f64| mach.kw.max(0.0) * (n / mach.rpm_base.max(1.0)).min(1.0) * SPINDLE_EFF;
    // Force and power grow roughly as fz^(1-mc), so this exponent inverts a limit.
    let ex = 1.0 / (1.0 - m.mc);

    if tool.kind == ToolKind::Drill {
        let depth = s.cut.ap.max(0.01);
        let vc_t = vc_base * 0.7 * coat_m * life_m;
        log.row(Step::VcBase(tool.material), vc_base, Unit::MPerMin, 0);
        log.factor(Step::Drilling, 0.7);
        log.factor(Step::Coating(tool.coating), coat_m);
        log.factor(Step::LifeTweak, life_m);
        log.total(Step::VcTarget, vc_t, Unit::MPerMin, 0);

        let n = clamp_rpm(vc_t * 1000.0 / (PI * d), &mut limiters);
        let vc = PI * d * n / 1000.0;
        let fn_t = m.cf * 2.5 * dterm(d) * tm_f * rigid_f * fin_m;
        log.row(Step::FnBase { cf: m.cf * 2.5 }, m.cf * 2.5 * dterm(d), Unit::MmPerRev, 3);
        log.factor(Step::FnFactors, tm_f * rigid_f * fin_m);
        log.total(Step::FnTarget, fn_t, Unit::MmPerRev, 3);

        let sk = 59.0f64.to_radians().sin(); // half of a 118° point angle
        let phys = |f: f64| {
            let vf = f * n;
            let hm = f / 2.0 * sk;
            let kc = m.kc * hm.powf(-m.mc);
            let pc = f * vc * d * kc / 240_000.0;
            let torque = pc * 9549.0 / n;
            let thrust = 0.5 * kc * d / 2.0 * f * sk;
            (vf, kc, pc, torque, thrust)
        };
        let pav = avail(n);
        let mut k = 1.0f64;
        let (vf0, _, pc0, _, _) = phys(fn_t);
        if vf0 > feed_max {
            k = k.min(feed_max / vf0);
            limiters.push(Limiter::FeedMax);
        }
        if s.cut.limit_power && pc0 > pav && pav > 0.0 {
            k = k.min((pav / pc0).powf(ex));
            limiters.push(Limiter::Power);
        }
        let f = fn_t * k;
        let (vf, kc, pc, torque, thrust) = phys(f);
        if k < 1.0 {
            let l = *limiters.last().unwrap_or(&Limiter::FeedMax);
            log.factor(Step::Reduction(l), k);
        }
        let ratio = depth / d;
        let peck = (ratio > 3.0).then(|| d * if g == Iso::N { 1.5 } else { 1.0 });
        if pc > pav * 1.001 {
            log.warn(Level::Crit, Msg::PowerShort { need_kw: pc / SPINDLE_EFF, avail_kw: pav / SPINDLE_EFF, rpm: n });
        }
        if let Some(q) = peck {
            log.warn(Level::Info, Msg::DeepHole { ratio, peck: q });
        }
        if ratio > 8.0 {
            log.warn(Level::Warn, Msg::Over8D);
        }
        rpm_warnings(&mut log, n, rpm_max, rpm_min, vc, vc_t);
        let (why, warnings) = log.sorted_warnings();
        return Outcome {
            kind: tool.kind,
            group: g,
            rpm: n,
            vc,
            vc_target: vc_t,
            feed: vf,
            fz: f,
            chip: f,
            chip_target: fn_t,
            thinning: 1.0,
            mrr: PI * d * d / 4.0 * vf / 1000.0,
            spindle_kw: pc / SPINDLE_EFF,
            avail_kw: pav / SPINDLE_EFF,
            torque,
            force: thrust,
            kc,
            deflection: None,
            deflection_limit: s.cut.deflection_limit,
            d,
            d_eff: d,
            ap: depth,
            ae: d,
            radial_ratio: 1.0,
            kappa: 59.0,
            z: 2,
            limiters,
            warnings,
            why,
            drill: Some(DrillInfo { thrust_n: thrust, time_s: depth / vf * 60.0, depth, peck }),
        };
    }

    // ---- milling
    let z = tool.z.max(1);
    let ap = s.cut.ap.max(0.001);
    let mut ae = s.cut.ae.max(0.001);
    let mut d_eff = d;
    let mut kappa = 90.0f64;
    let mut area = None;
    match tool.kind {
        ToolKind::Ball => {
            let r = d / 2.0;
            if ap < r {
                d_eff = 2.0 * (ap * (d - ap)).sqrt();
                kappa = (d_eff / d).asin().to_degrees();
            }
        }
        ToolKind::Bull => {
            let r = tool.corner_r.clamp(0.0, d / 2.0);
            if r > 0.0 && ap < r {
                d_eff = d - 2.0 * r + 2.0 * (ap * (2.0 * r - ap)).sqrt();
                kappa = ((r - ap) / r).acos().to_degrees();
            }
        }
        ToolKind::VBit => {
            let half = (tool.v_angle / 2.0).clamp(1.0, 89.0);
            let d0 = tool.tip_d.max(0.0);
            let full = d0 + 2.0 * ap * half.to_radians().tan();
            d_eff = full.min(d);
            kappa = 90.0 - half;
            ae = d_eff;
            area = Some(ap * (d0 + ap * half.to_radians().tan()));
            if full > d {
                log.warn(Level::Warn, Msg::VBitTooDeep { ap });
            }
        }
        ToolKind::Face => kappa = tool.kappa.clamp(10.0, 90.0),
        _ => {}
    }
    let d_eff = d_eff.max(0.05);
    let ae_e = ae.min(d_eff);
    let rr = ae_e / d_eff;
    let area = area.unwrap_or(ap * ae_e);

    // Narrow cuts leave the edge time to cool, so speed can rise; a full slot runs hotter.
    let ae_boost = if rr >= 0.99 {
        0.9
    } else if rr >= 0.5 {
        1.0
    } else {
        1.0 + 0.4 * ((0.5 - rr) / 0.4).min(1.0)
    };
    let vc_t = vc_base * coat_m * ae_boost * life_m;
    log.row(Step::VcBase(tool.material), vc_base, Unit::MPerMin, 0);
    log.factor(Step::Coating(tool.coating), coat_m);
    if rr >= 0.99 {
        log.factor(Step::FullSlot, ae_boost);
    } else {
        log.factor(Step::Engagement { pct: rr * 100.0, cooler: rr < 0.5 }, ae_boost);
    }
    log.factor(Step::LifeTweak, life_m);
    log.total(Step::VcTarget, vc_t, Unit::MPerMin, 0);
    if d_eff < d - 1e-6 {
        log.row(Step::EffectiveDiameter, d_eff, Unit::Mm, 2);
    }

    let n = clamp_rpm(vc_t * 1000.0 / (PI * d_eff), &mut limiters);
    let vc = PI * d_eff * n / 1000.0;

    let fz_base = if tool.kind == ToolKind::Face { g.face_fz() } else { m.cf * dterm(d) };
    let slot = rr >= 0.99;
    let slot_f = if slot { 0.85 } else { 1.0 };
    let fz_t = fz_base * tm_f * rigid_f * fin_m * slot_f;
    if tool.kind == ToolKind::Face {
        log.row(Step::FzBaseInsert, fz_base, Unit::Mm, 3);
    } else {
        log.row(Step::FzBase { cf: m.cf }, fz_base, Unit::Mm, 4);
    }
    log.factor(Step::FzFactors { tool: tm_f, rigidity: rigid_f, tweak: fin_m, slot }, tm_f * rigid_f * fin_m * slot_f);
    log.total(Step::ChipTarget, fz_t, Unit::Mm, 4);

    let rad_t = if rr < 0.5 { 1.0 / (2.0 * (rr - rr * rr).sqrt()) } else { 1.0 };
    let lead_t = 1.0 / kappa.to_radians().sin();
    let thin = if s.cut.chip_thinning { (rad_t * lead_t).min(5.0) } else { 1.0 };
    if s.cut.chip_thinning && thin > 1.001 {
        if rad_t > 1.001 {
            log.factor(Step::RadialThinning, rad_t);
        }
        if lead_t > 1.001 {
            let step = if tool.kind == ToolKind::Ball { Step::BallThinning } else { Step::LeadThinning { kappa } };
            log.factor(step, lead_t);
        }
        if rad_t * lead_t > 5.0 {
            log.row(Step::ThinningCapped, 5.0, Unit::Factor, 0);
        }
    }
    let fz_p = fz_t * thin;

    let stick = tool.stickout.max(0.1);
    let flute = tool.flute_len.max(0.1);
    let ds = if tool.shank_d > 0.0 { tool.shank_d } else { d };
    let e = tool.material.young();
    let phi = (1.0 - 2.0 * rr).clamp(-1.0, 1.0).acos(); // engagement angle
    let sin_k = kappa.to_radians().sin();
    let is_face = tool.kind == ToolKind::Face;
    let phys = |fz: f64| {
        let vf = fz * z as f64 * n;
        let hm = (2.0 * sin_k * ae_e * fz / (d_eff * phi)).max(1e-5);
        let kc = m.kc * hm.powf(-m.mc);
        let pc = area * vf * kc / 6.0e7;
        let torque = pc * 9549.0 / n;
        let force = 2000.0 * torque / d_eff;
        let defl = if is_face { 0.0 } else { deflection(force, stick, flute, d, ds, e) };
        (vf, kc, pc, torque, force, defl)
    };
    let pav = avail(n);
    let dlim = s.cut.deflection_limit.max(0.0005);
    let (vf0, _, pc0, _, _, defl0) = phys(fz_p);
    let mut reasons: Vec<(Limiter, f64)> = Vec::new();
    if vf0 > feed_max {
        reasons.push((Limiter::FeedMax, feed_max / vf0));
    }
    if s.cut.limit_power && pc0 > pav && pav > 0.0 {
        reasons.push((Limiter::Power, (pav / pc0).powf(ex)));
    }
    if s.cut.limit_deflection && !is_face && defl0 > dlim {
        reasons.push((Limiter::Deflection, (dlim / defl0).powf(ex)));
    }
    reasons.sort_by(|a, b| a.1.total_cmp(&b.1));
    let k = reasons.first().map(|r| r.1).unwrap_or(1.0).min(1.0);
    if let Some((l, _)) = reasons.first() {
        limiters.push(*l);
        log.factor(Step::Reduction(*l), k);
    }
    let fz = fz_p * k;
    let (vf, kc, pc, torque, force, defl) = phys(fz);
    log.total(Step::FzProgrammed, fz, Unit::Mm, 4);
    let chip = fz_t * k;

    // warnings
    let min_chip = if metal {
        if tool.material == ToolMaterial::Carbide { 0.008 } else { 0.005 }
    } else {
        0.02
    };
    if chip < min_chip {
        log.warn(if metal { Level::Warn } else { Level::Crit }, Msg::ThinChip { chip, metal });
    }
    if k < 0.6 {
        match reasons.first().map(|r| r.0) {
            Some(Limiter::Deflection) => log.warn(Level::Warn, Msg::DeflectionHalvesFeed),
            Some(Limiter::Power) => log.warn(Level::Warn, Msg::PowerHalvesFeed),
            _ => {}
        }
    }
    if pc > pav * 1.001 {
        log.warn(Level::Crit, Msg::PowerShort { need_kw: pc / SPINDLE_EFF, avail_kw: pav / SPINDLE_EFF, rpm: n });
    }
    if !is_face {
        if defl > dlim * 1.001 {
            log.warn(Level::Warn, Msg::Deflection { um: defl * 1000.0, limit_um: dlim * 1000.0 });
        }
        if ap > flute {
            log.warn(Level::Crit, Msg::ApOverFlute { flute });
        }
        if stick / d > 4.0 {
            log.warn(Level::Info, Msg::LongStickout { ratio: stick / d });
        }
        if stick < flute {
            log.warn(Level::Info, Msg::StickoutShorterThanFlute);
        }
        if slot && g == Iso::N && z > 3 {
            log.warn(Level::Warn, Msg::TooManyFlutesInAlSlot { z });
        }
    }
    if slot && ap > d && metal {
        log.warn(Level::Warn, Msg::DeepSlot);
    }
    rpm_warnings(&mut log, n, rpm_max, rpm_min, vc, vc_t);
    let (why, warnings) = log.sorted_warnings();

    Outcome {
        kind: tool.kind,
        group: g,
        rpm: n,
        vc,
        vc_target: vc_t,
        feed: vf,
        fz,
        chip,
        chip_target: fz_t,
        thinning: thin,
        mrr: area * vf / 1000.0,
        spindle_kw: pc / SPINDLE_EFF,
        avail_kw: pav / SPINDLE_EFF,
        torque,
        force,
        kc,
        deflection: (!is_face).then_some(defl),
        deflection_limit: dlim,
        d,
        d_eff,
        ap,
        ae: ae_e,
        radial_ratio: rr,
        kappa,
        z,
        limiters,
        warnings,
        why,
        drill: None,
    }
}

fn rpm_warnings(log: &mut Log, n: f64, rpm_max: f64, rpm_min: f64, vc: f64, vc_t: f64) {
    if n >= rpm_max && vc < vc_t * 0.7 {
        log.warn(Level::Info, Msg::RpmCapped { vc, pct: vc / vc_t * 100.0 });
    }
    if n <= rpm_min.max(1.0) && vc > vc_t * 1.3 {
        log.warn(Level::Crit, Msg::RpmMinTooHigh { vc, target: vc_t });
    }
}

/// Variable swept by the curve chart.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SweepVar {
    /// Radial engagement ae as % of Ø.
    Radial,
    /// Axial depth ap in mm.
    Axial,
}

#[derive(Clone, Copy, Debug)]
pub struct SweepPoint {
    pub x: f64,
    pub mrr: f64,
    pub spindle_kw: f64,
    pub avail_kw: f64,
    pub feed: f64,
    pub limited: bool,
}

/// Recomputes the setup while sweeping ae (% of Ø) or ap (mm), everything else fixed.
pub fn sweep(s: &Setup, var: SweepVar, steps: usize) -> Vec<SweepPoint> {
    let steps = steps.max(2);
    let mut out = Vec::with_capacity(steps);
    let d = s.tool.d.max(0.1);
    let ap_max = match s.tool.kind {
        ToolKind::Face => (s.cut.ap * 3.0).max(1.0),
        ToolKind::Drill => (d * 10.0).max(s.cut.ap),
        _ => s.tool.flute_len.max(s.cut.ap).max(0.1),
    };
    for i in 0..steps {
        let t = i as f64 / (steps - 1) as f64;
        let mut c = s.clone();
        let x = match var {
            SweepVar::Radial => {
                let pct = 2.0 + t * 98.0;
                c.cut.ae = d * pct / 100.0;
                pct
            }
            SweepVar::Axial => {
                let ap = ap_max * (0.02 + 0.98 * t);
                c.cut.ap = ap;
                ap
            }
        };
        let o = compute(&c);
        out.push(SweepPoint {
            x,
            mrr: o.mrr,
            spindle_kw: o.spindle_kw,
            avail_kw: o.avail_kw,
            feed: o.feed,
            limited: o.limiters.iter().any(|l| matches!(l, Limiter::Power | Limiter::Deflection | Limiter::FeedMax)),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() <= tol * b.abs().max(1e-9)
    }

    #[test]
    fn aluminium_on_hobby_router() {
        // 6 mm 3-flute ZrN carbide in 6061 on a router: target Vc exceeds max rpm → 24 000 rpm.
        let o = compute(&Setup::default());
        assert_eq!(o.rpm, 24000.0);
        assert!(o.limiters.contains(&Limiter::RpmMax));
        // target fz 0.06 · 0.6 = 0.036; thinning at 20 % ae = 1.25 → programmed fz 0.045
        assert!(close(o.chip_target, 0.036, 1e-6));
        assert!(close(o.thinning, 1.25, 1e-6));
        assert!(close(o.feed, 0.045 * 3.0 * 24000.0, 1e-6));
        assert!(o.power_ok());
        let defl = o.deflection.unwrap();
        assert!(defl > 0.010 && defl < 0.025, "deflection {defl}");
    }

    #[test]
    fn radial_chip_thinning() {
        let mut s = Setup::default();
        s.cut.ae = s.tool.d * 0.5;
        assert!(close(compute(&s).thinning, 1.0, 1e-9));
        s.cut.ae = s.tool.d * 0.1;
        // 1 / (2·√(0.1 − 0.01)) = 1.6667
        assert!(close(compute(&s).thinning, 1.0 / (2.0 * 0.09f64.sqrt()), 1e-9));
    }

    #[test]
    fn power_limit_is_respected() {
        let mut s = Setup::default();
        s.material_id = "ss304".into();
        s.tool.coating = Coating::AlTiN;
        s.tool.d = 12.0;
        s.tool.shank_d = 12.0;
        s.tool.flute_len = 30.0;
        s.tool.stickout = 40.0;
        s.machine = Machine::from_preset(MachinePreset::Bench);
        s.apply_strategy(Strategy::Slot);
        let o = compute(&s);
        assert!(o.spindle_kw <= o.avail_kw * 1.01, "{} > {}", o.spindle_kw, o.avail_kw);
    }

    #[test]
    fn deflection_limit_is_respected() {
        let mut s = Setup::default();
        s.material_id = "c45".into();
        s.tool.stickout = 45.0;
        s.machine = Machine::from_preset(MachinePreset::Vmc);
        s.cut.ae = 3.0;
        let o = compute(&s);
        assert!(o.deflection.unwrap() <= s.cut.deflection_limit * 1.01);
        assert!(o.limiters.contains(&Limiter::Deflection));
    }

    #[test]
    fn ball_nose_effective_diameter() {
        let mut s = Setup::default();
        s.tool.kind = ToolKind::Ball;
        s.tool.d = 10.0;
        s.cut.ap = 1.0;
        // 2·√(1·9) = 6
        assert!(close(compute(&s).d_eff, 6.0, 1e-9));
    }

    #[test]
    fn deep_hole_uses_g83() {
        let mut s = Setup::default();
        s.tool.kind = ToolKind::Drill;
        s.material_id = "c45".into();
        s.machine = Machine::from_preset(MachinePreset::Vmc);
        s.cut.ap = 30.0;
        let o = compute(&s);
        assert!(o.gcode().contains("G83"));
        assert!(o.warnings.iter().any(|w| matches!(w.msg, Msg::DeepHole { .. })));
        let pc_check = o.fz * o.vc * o.d * o.kc / 240_000.0 / SPINDLE_EFF;
        assert!(close(o.spindle_kw, pc_check, 1e-9));
    }

    #[test]
    fn machine_max_feed() {
        let mut s = Setup::default();
        s.machine.feed_max = 1000.0;
        let o = compute(&s);
        assert!(o.feed <= 1000.0 + 1e-6);
        assert!(o.limiters.contains(&Limiter::FeedMax));
    }

    #[test]
    fn sweep_returns_finite_points() {
        let pts = sweep(&Setup::default(), SweepVar::Radial, 30);
        assert_eq!(pts.len(), 30);
        assert!(pts.iter().all(|p| p.mrr.is_finite() && p.spindle_kw.is_finite()));
    }

    #[test]
    fn json_round_trip() {
        let s = Setup::default();
        let j = serde_json::to_string(&s).unwrap();
        let back: Setup = serde_json::from_str(&j).unwrap();
        assert_eq!(s, back);
    }
}
