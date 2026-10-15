//! Reference tables: materials, coatings, machine presets, cutting strategies.
//!
//! Values are typical handbook figures, meant as a starting point to be checked
//! against the tool manufacturer's catalogue. Display names live in `i18n`.

use serde::{Deserialize, Serialize};

/// ISO 513 material groups, plus plastics (O) and wood (W).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Iso {
    N,
    P,
    M,
    K,
    S,
    H,
    O,
    W,
}

impl Iso {
    pub const ALL: [Iso; 8] = [
        Iso::N,
        Iso::P,
        Iso::M,
        Iso::K,
        Iso::S,
        Iso::H,
        Iso::O,
        Iso::W,
    ];

    pub fn letter(self) -> &'static str {
        match self {
            Iso::N => "N",
            Iso::P => "P",
            Iso::M => "M",
            Iso::K => "K",
            Iso::S => "S",
            Iso::H => "H",
            Iso::O => "O",
            Iso::W => "W",
        }
    }

    /// Group colour following the ISO 513 convention (RGB).
    pub fn rgb(self) -> [u8; 3] {
        match self {
            Iso::N => [46, 139, 87],
            Iso::P => [31, 95, 168],
            Iso::M => [217, 164, 0],
            Iso::K => [192, 57, 43],
            Iso::S => [199, 106, 16],
            Iso::H => [107, 118, 130],
            Iso::O => [122, 79, 160],
            Iso::W => [138, 106, 62],
        }
    }

    pub fn is_metal(self) -> bool {
        !matches!(self, Iso::O | Iso::W)
    }
    pub fn is_soft(self) -> bool {
        matches!(self, Iso::N | Iso::O | Iso::W)
    }
    pub fn is_hard(self) -> bool {
        matches!(self, Iso::M | Iso::S | Iso::H)
    }

    /// Typical feed per tooth of an indexable face mill (mm).
    pub fn face_fz(self) -> f64 {
        match self {
            Iso::N | Iso::P => 0.15,
            Iso::M => 0.12,
            Iso::K => 0.18,
            Iso::S => 0.10,
            Iso::H => 0.08,
            Iso::O | Iso::W => 0.20,
        }
    }

    /// Typical rake angle of a solid carbide end mill for this group (degrees).
    /// Sandvik Coromant: power changes 1 % per degree of rake, and kc1 is given for 0°.
    pub fn rake_deg(self) -> f64 {
        match self {
            Iso::N | Iso::O | Iso::W => 15.0,
            Iso::P | Iso::M | Iso::S => 8.0,
            Iso::K => 5.0,
            Iso::H => 0.0,
        }
    }

    /// Twist drill cutting speed as a fraction of the end mill one. From the Kennametal
    /// GOdrill and GOmill PRO start values for the same group.
    pub fn drill_vc(self) -> f64 {
        match self {
            Iso::P => 0.5,
            Iso::M => 0.35,
            Iso::K => 0.9,
            Iso::N => 0.55,
            Iso::S => 0.55,
            Iso::H => 0.2,
            Iso::O | Iso::W => 0.7,
        }
    }

    /// Twist drill feed per rev factor on top of 2.5·cf·Ø (Kennametal GOdrill ranges:
    /// cast iron takes more, stainless less).
    pub fn drill_fn(self) -> f64 {
        match self {
            Iso::K => 1.5,
            Iso::M => 0.8,
            _ => 1.0,
        }
    }

    /// Recommended ramp angle (degrees).
    pub fn ramp_deg(self) -> f64 {
        match self {
            Iso::N => 3.0,
            Iso::P | Iso::K => 2.0,
            Iso::M | Iso::S => 1.5,
            Iso::H => 1.0,
            Iso::O | Iso::W => 5.0,
        }
    }
}

/// Cutting speed factor for a hardness different from the nominal one, as points
/// (hardness difference, factor). From the Sandvik Coromant milling guide.
#[derive(Debug)]
pub struct HardnessCurve {
    /// Hardness in HRC instead of HB.
    pub hrc: bool,
    pub points: &'static [(f64, f64)],
}

impl HardnessCurve {
    /// Factor for a hardness `delta` above (+) or below (−) nominal; clamped to the table.
    pub fn factor(&self, delta: f64) -> f64 {
        let p = self.points;
        if delta <= p[0].0 {
            return p[0].1;
        }
        for w in p.windows(2) {
            let ((x0, y0), (x1, y1)) = (w[0], w[1]);
            if delta <= x1 {
                return y0 + (y1 - y0) * (delta - x0) / (x1 - x0);
            }
        }
        p[p.len() - 1].1
    }

    /// Hardness range the table covers, around a nominal value.
    pub fn range(&self, nominal: f64) -> (f64, f64) {
        let p = self.points;
        (nominal + p[0].0, nominal + p[p.len() - 1].0)
    }
}

/// Unalloyed steel (CMC 01).
const HB_UNALLOYED: HardnessCurve = HardnessCurve {
    hrc: false,
    points: &[(-20.0, 1.07), (0.0, 1.0), (20.0, 0.95), (40.0, 0.90)],
};
/// Low-alloy steel (CMC 02).
const HB_LOW_ALLOY: HardnessCurve = HardnessCurve {
    hrc: false,
    points: &[
        (-80.0, 1.26),
        (-60.0, 1.18),
        (-40.0, 1.12),
        (-20.0, 1.05),
        (0.0, 1.0),
        (20.0, 0.94),
        (40.0, 0.91),
        (60.0, 0.86),
        (80.0, 0.83),
    ],
};
/// Stainless steel (CMC 05).
const HB_STAINLESS: HardnessCurve = HardnessCurve {
    hrc: false,
    points: &[
        (-40.0, 1.21),
        (-20.0, 1.10),
        (0.0, 1.0),
        (20.0, 0.91),
        (40.0, 0.85),
        (60.0, 0.79),
        (80.0, 0.75),
    ],
};
/// Grey cast iron (CMC 08).
const HB_GREY_IRON: HardnessCurve = HardnessCurve {
    hrc: false,
    points: &[
        (-40.0, 1.25),
        (-20.0, 1.10),
        (0.0, 1.0),
        (20.0, 0.92),
        (40.0, 0.86),
        (60.0, 0.80),
    ],
};
/// Nodular cast iron (CMC 09).
const HB_NODULAR: HardnessCurve = HardnessCurve {
    hrc: false,
    points: &[
        (-40.0, 1.07),
        (-20.0, 1.03),
        (0.0, 1.0),
        (20.0, 0.97),
        (40.0, 0.95),
        (60.0, 0.93),
        (80.0, 0.91),
    ],
};
/// Heat resistant superalloys (CMC 20).
const HB_SUPERALLOY: HardnessCurve = HardnessCurve {
    hrc: false,
    points: &[
        (-80.0, 1.26),
        (-40.0, 1.11),
        (0.0, 1.0),
        (40.0, 0.90),
        (80.0, 0.82),
    ],
};
/// Hardened steel (CMC 04), in HRC.
const HRC_HARDENED: HardnessCurve = HardnessCurve {
    hrc: true,
    points: &[
        (-6.0, 1.10),
        (-3.0, 1.02),
        (0.0, 1.0),
        (3.0, 0.96),
        (6.0, 0.93),
        (9.0, 0.90),
    ],
};

#[derive(Clone, Copy, Debug)]
pub struct Material {
    pub id: &'static str,
    pub group: Iso,
    /// Base cutting speed with carbide (m/min).
    pub vc_carbide: f64,
    /// Base cutting speed with HSS (m/min); 0 = HSS not suitable.
    pub vc_hss: f64,
    /// Feed per tooth ≈ cf · Ø (mm).
    pub cf: f64,
    /// Specific cutting force kc1.1 (N/mm²) for a 1 mm chip.
    pub kc: f64,
    /// Kienzle exponent.
    pub mc: f64,
    /// Nominal hardness (HB, or HRC for hardened steel) the cutting speed refers to.
    pub hardness: f64,
    /// How cutting speed changes with hardness; None where no table is available.
    pub hardness_curve: Option<&'static HardnessCurve>,
}

const fn mat(
    id: &'static str,
    group: Iso,
    vc_carbide: f64,
    vc_hss: f64,
    cf: f64,
    kc: f64,
    mc: f64,
    hardness: f64,
    hardness_curve: Option<&'static HardnessCurve>,
) -> Material {
    Material {
        id,
        group,
        vc_carbide,
        vc_hss,
        cf,
        kc,
        mc,
        hardness,
        hardness_curve,
    }
}

// Columns: Vc carbide, Vc HSS (m/min), cf (fz ≈ cf·Ø), kc1.1 (N/mm²), mc, nominal hardness.
// kc1.1 checked against the Sandvik Coromant power constants (milling guide, CMC table);
// Vc and cf of P, M, K, S, H checked against the Kennametal GOmill PRO tables (start
// values, Ø10, side milling at 0.4·Ø, divided by the AlTiN factor for the uncoated base).
#[rustfmt::skip]
pub const MATERIALS: &[Material] = &[
    mat("al6061",   Iso::N, 450.0, 120.0, 0.010,  650.0, 0.25,  95.0, None),
    mat("al7075",   Iso::N, 400.0, 110.0, 0.010,  700.0, 0.25, 150.0, None),
    mat("alcast",   Iso::N, 300.0,  80.0, 0.008,  750.0, 0.25,  90.0, None),
    mat("brass",    Iso::N, 250.0,  90.0, 0.008,  700.0, 0.18, 110.0, None),
    mat("bronze",   Iso::N, 180.0,  50.0, 0.007, 1500.0, 0.20, 120.0, None),
    mat("copper",   Iso::N, 200.0,  60.0, 0.006, 1500.0, 0.20,  90.0, None),
    mat("s235",     Iso::P, 130.0,  30.0, 0.007, 1500.0, 0.25, 130.0, Some(&HB_UNALLOYED)),
    mat("c45",      Iso::P, 125.0,  25.0, 0.007, 1600.0, 0.25, 200.0, Some(&HB_UNALLOYED)),
    mat("42crmo4",  Iso::P, 100.0,  18.0, 0.006, 2000.0, 0.25, 280.0, Some(&HB_LOW_ALLOY)),
    mat("p20",      Iso::P, 100.0,  18.0, 0.006, 2000.0, 0.25, 300.0, Some(&HB_LOW_ALLOY)),
    mat("ss304",    Iso::M,  80.0,  18.0, 0.006, 2000.0, 0.21, 180.0, Some(&HB_STAINLESS)),
    mat("ss316",    Iso::M,  75.0,  15.0, 0.0055, 2150.0, 0.21, 180.0, Some(&HB_STAINLESS)),
    mat("ss174",    Iso::M,  60.0,  12.0, 0.0048, 2600.0, 0.21, 330.0, Some(&HB_STAINLESS)),
    mat("gg25",     Iso::K, 105.0,  25.0, 0.007, 1100.0, 0.28, 220.0, Some(&HB_GREY_IRON)),
    mat("ggg50",    Iso::K,  95.0,  20.0, 0.006, 1300.0, 0.28, 200.0, Some(&HB_NODULAR)),
    mat("ti64",     Iso::S,  45.0,  12.0, 0.0045, 1400.0, 0.23, 330.0, None),
    mat("in718",    Iso::S,  30.0,   6.0, 0.0032, 2900.0, 0.25, 380.0, Some(&HB_SUPERALLOY)),
    mat("h50",      Iso::H,  70.0,   0.0, 0.0035, 4000.0, 0.25,  50.0, Some(&HRC_HARDENED)),
    mat("pom",      Iso::O, 350.0, 200.0, 0.012,  300.0, 0.30,   0.0, None),
    mat("pmma",     Iso::O, 300.0, 180.0, 0.010,  320.0, 0.30,   0.0, None),
    mat("hdpe",     Iso::O, 350.0, 200.0, 0.014,  150.0, 0.30,   0.0, None),
    mat("g10",      Iso::O, 180.0,   0.0, 0.006,  450.0, 0.30,   0.0, None),
    mat("hardwood", Iso::W, 600.0, 300.0, 0.016,  120.0, 0.30,   0.0, None),
    mat("softwood", Iso::W, 700.0, 350.0, 0.018,   80.0, 0.30,   0.0, None),
    mat("mdf",      Iso::W, 600.0, 300.0, 0.018,   90.0, 0.30,   0.0, None),
];

pub fn material(id: &str) -> &'static Material {
    MATERIALS
        .iter()
        .find(|m| m.id == id)
        .unwrap_or(&MATERIALS[0])
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Coating {
    None,
    TiN,
    TiCN,
    AlTiN,
    ZrN,
    Diamond,
}

impl Coating {
    pub const ALL: [Coating; 6] = [
        Coating::None,
        Coating::TiN,
        Coating::TiCN,
        Coating::AlTiN,
        Coating::ZrN,
        Coating::Diamond,
    ];

    /// Cutting speed multiplier per material group.
    pub fn vc_factor(self, g: Iso) -> f64 {
        use Iso::*;
        match (self, g) {
            (Coating::TiN, P | M | K) => 1.1,
            (Coating::TiN, S) => 1.05,
            (Coating::TiCN, P | K) => 1.2,
            (Coating::TiCN, M) => 1.15,
            (Coating::TiCN, S) => 1.1,
            (Coating::TiCN, H) => 1.05,
            (Coating::AlTiN, N) => 0.85,
            (Coating::AlTiN, P) => 1.35,
            (Coating::AlTiN, M | K | H) => 1.3,
            (Coating::AlTiN, S) => 1.25,
            (Coating::ZrN, N) => 1.2,
            (Coating::ZrN, O) => 1.1,
            (Coating::Diamond, N) => 1.4,
            (Coating::Diamond, O | W) => 1.3,
            _ => 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolMaterial {
    Carbide,
    HssCo,
    Hss,
}

impl ToolMaterial {
    pub const ALL: [ToolMaterial; 3] = [
        ToolMaterial::Carbide,
        ToolMaterial::HssCo,
        ToolMaterial::Hss,
    ];

    /// Young's modulus (N/mm²) used for the deflection estimate.
    pub fn young(self) -> f64 {
        match self {
            ToolMaterial::Carbide => 620_000.0,
            _ => 210_000.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolKind {
    Flat,
    Ball,
    Bull,
    VBit,
    Face,
    Drill,
}

impl ToolKind {
    pub const ALL: [ToolKind; 6] = [
        ToolKind::Flat,
        ToolKind::Ball,
        ToolKind::Bull,
        ToolKind::VBit,
        ToolKind::Face,
        ToolKind::Drill,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rigidity {
    Low,
    Medium,
    High,
}

impl Rigidity {
    pub const ALL: [Rigidity; 3] = [Rigidity::Low, Rigidity::Medium, Rigidity::High];

    /// Chip load multiplier applied to metals.
    pub fn factor(self) -> f64 {
        match self {
            Rigidity::Low => 0.6,
            Rigidity::Medium => 0.85,
            Rigidity::High => 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MachinePreset {
    Router,
    Bench,
    Vmc,
    Custom,
}

impl MachinePreset {
    pub const ALL: [MachinePreset; 4] = [
        MachinePreset::Router,
        MachinePreset::Bench,
        MachinePreset::Vmc,
        MachinePreset::Custom,
    ];
}

/// Cut strategies used by the preset buttons and the comparison chart.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Strategy {
    Slot,
    Rough,
    Hsm,
    Finish,
}

impl Strategy {
    pub const ALL: [Strategy; 4] = [
        Strategy::Slot,
        Strategy::Rough,
        Strategy::Hsm,
        Strategy::Finish,
    ];
}
