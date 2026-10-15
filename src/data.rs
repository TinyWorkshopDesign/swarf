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
    pub const ALL: [Iso; 8] = [Iso::N, Iso::P, Iso::M, Iso::K, Iso::S, Iso::H, Iso::O, Iso::W];

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
}

const fn mat(id: &'static str, group: Iso, vc_carbide: f64, vc_hss: f64, cf: f64, kc: f64, mc: f64) -> Material {
    Material { id, group, vc_carbide, vc_hss, cf, kc, mc }
}

pub const MATERIALS: &[Material] = &[
    mat("al6061", Iso::N, 450.0, 120.0, 0.010, 700.0, 0.25),
    mat("al7075", Iso::N, 400.0, 110.0, 0.010, 800.0, 0.25),
    mat("alcast", Iso::N, 300.0, 80.0, 0.008, 750.0, 0.25),
    mat("brass", Iso::N, 250.0, 90.0, 0.008, 780.0, 0.18),
    mat("bronze", Iso::N, 180.0, 50.0, 0.007, 1300.0, 0.20),
    mat("copper", Iso::N, 200.0, 60.0, 0.006, 1100.0, 0.20),
    mat("s235", Iso::P, 150.0, 30.0, 0.006, 1500.0, 0.25),
    mat("c45", Iso::P, 130.0, 25.0, 0.0055, 1700.0, 0.25),
    mat("42crmo4", Iso::P, 100.0, 18.0, 0.005, 2000.0, 0.25),
    mat("p20", Iso::P, 100.0, 18.0, 0.005, 2000.0, 0.25),
    mat("ss304", Iso::M, 90.0, 18.0, 0.0045, 2000.0, 0.21),
    mat("ss316", Iso::M, 80.0, 15.0, 0.0045, 2150.0, 0.21),
    mat("ss174", Iso::M, 70.0, 12.0, 0.004, 2400.0, 0.21),
    mat("gg25", Iso::K, 130.0, 25.0, 0.006, 1100.0, 0.28),
    mat("ggg50", Iso::K, 110.0, 20.0, 0.0055, 1300.0, 0.28),
    mat("ti64", Iso::S, 50.0, 12.0, 0.004, 1450.0, 0.23),
    mat("in718", Iso::S, 30.0, 6.0, 0.003, 2900.0, 0.25),
    mat("h50", Iso::H, 70.0, 0.0, 0.0025, 3500.0, 0.25),
    mat("pom", Iso::O, 350.0, 200.0, 0.012, 300.0, 0.30),
    mat("pmma", Iso::O, 300.0, 180.0, 0.010, 320.0, 0.30),
    mat("hdpe", Iso::O, 350.0, 200.0, 0.014, 150.0, 0.30),
    mat("g10", Iso::O, 180.0, 0.0, 0.006, 450.0, 0.30),
    mat("hardwood", Iso::W, 600.0, 300.0, 0.016, 120.0, 0.30),
    mat("softwood", Iso::W, 700.0, 350.0, 0.018, 80.0, 0.30),
    mat("mdf", Iso::W, 600.0, 300.0, 0.018, 90.0, 0.30),
];

pub fn material(id: &str) -> &'static Material {
    MATERIALS.iter().find(|m| m.id == id).unwrap_or(&MATERIALS[0])
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
    pub const ALL: [Coating; 6] = [Coating::None, Coating::TiN, Coating::TiCN, Coating::AlTiN, Coating::ZrN, Coating::Diamond];

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
    pub const ALL: [ToolMaterial; 3] = [ToolMaterial::Carbide, ToolMaterial::HssCo, ToolMaterial::Hss];

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
    pub const ALL: [ToolKind; 6] = [ToolKind::Flat, ToolKind::Ball, ToolKind::Bull, ToolKind::VBit, ToolKind::Face, ToolKind::Drill];
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
    pub const ALL: [MachinePreset; 4] = [MachinePreset::Router, MachinePreset::Bench, MachinePreset::Vmc, MachinePreset::Custom];
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
    pub const ALL: [Strategy; 4] = [Strategy::Slot, Strategy::Rough, Strategy::Hsm, Strategy::Finish];
}
