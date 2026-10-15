//! Display units. The engine works in metric; these convert for display and input.

use serde::{Deserialize, Serialize};

pub const MM_PER_IN: f64 = 25.4;
const FT_PER_M: f64 = 1.0 / 0.3048;
const CM3_PER_IN3: f64 = 16.387_064;
const HP_PER_KW: f64 = 1.0 / 0.745_699_872;
const LBF_PER_N: f64 = 1.0 / 4.448_221_615;
const LBFFT_PER_NM: f64 = 1.0 / 1.355_817_948;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Units {
    #[default]
    Metric,
    Inch,
}

impl Units {
    /// Inch for the regions that machine in inches (BCP-47 tag such as "en-US").
    pub fn from_locale(tag: &str) -> Units {
        let region = tag
            .split(['-', '_', '.'])
            .skip(1)
            .find(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_alphabetic()))
            .unwrap_or("")
            .to_ascii_uppercase();
        match region.as_str() {
            "US" | "LR" | "MM" => Units::Inch,
            _ => Units::Metric,
        }
    }

    pub fn inch(self) -> bool {
        self == Units::Inch
    }

    fn k(self, metric_to_inch: f64) -> f64 {
        if self.inch() { metric_to_inch } else { 1.0 }
    }

    /// Length: mm or in.
    pub fn len(self, mm: f64) -> f64 {
        mm / self.k(MM_PER_IN)
    }
    pub fn from_len(self, v: f64) -> f64 {
        v * self.k(MM_PER_IN)
    }
    pub fn len_unit(self) -> &'static str {
        if self.inch() { "in" } else { "mm" }
    }
    /// Decimals for a length shown with `dec` decimals in mm.
    pub fn len_dec(self, dec: usize) -> usize {
        if self.inch() { (dec + 1).max(3) } else { dec }
    }

    /// Small lengths (deflection, scallop): µm or in.
    pub fn micro(self, um: f64) -> f64 {
        if self.inch() {
            um / 1000.0 / MM_PER_IN
        } else {
            um
        }
    }
    pub fn from_micro(self, v: f64) -> f64 {
        if self.inch() {
            v * 1000.0 * MM_PER_IN
        } else {
            v
        }
    }
    pub fn micro_unit(self) -> &'static str {
        if self.inch() { "in" } else { "µm" }
    }
    pub fn micro_dec(self, dec: usize) -> usize {
        if self.inch() { dec + 4 } else { dec }
    }

    /// Surface roughness: µm or µin.
    pub fn rough(self, um: f64) -> f64 {
        um * self.k(1000.0 / MM_PER_IN)
    }
    pub fn rough_unit(self) -> &'static str {
        if self.inch() { "µin" } else { "µm" }
    }

    /// Cutting speed: m/min or SFM (ft/min).
    pub fn speed(self, m_min: f64) -> f64 {
        m_min * self.k(FT_PER_M)
    }
    pub fn from_speed(self, v: f64) -> f64 {
        v / self.k(FT_PER_M)
    }
    pub fn speed_unit(self) -> &'static str {
        if self.inch() { "SFM" } else { "m/min" }
    }

    /// Table feed: mm/min or in/min.
    pub fn feed(self, mm_min: f64) -> f64 {
        self.len(mm_min)
    }
    pub fn from_feed(self, v: f64) -> f64 {
        self.from_len(v)
    }
    pub fn feed_unit(self) -> &'static str {
        if self.inch() { "in/min" } else { "mm/min" }
    }
    pub fn feed_dec(self) -> usize {
        if self.inch() { 1 } else { 0 }
    }

    /// Removal rate: cm³/min or in³/min.
    pub fn mrr(self, cm3_min: f64) -> f64 {
        cm3_min / self.k(CM3_PER_IN3)
    }
    pub fn mrr_unit(self) -> &'static str {
        if self.inch() { "in³/min" } else { "cm³/min" }
    }

    /// Power: kW or hp.
    pub fn power(self, kw: f64) -> f64 {
        kw * self.k(HP_PER_KW)
    }
    pub fn from_power(self, v: f64) -> f64 {
        v / self.k(HP_PER_KW)
    }
    pub fn power_unit(self) -> &'static str {
        if self.inch() { "hp" } else { "kW" }
    }

    /// Torque: N·m or lbf·ft.
    pub fn torque(self, nm: f64) -> f64 {
        nm * self.k(LBFFT_PER_NM)
    }
    pub fn torque_unit(self) -> &'static str {
        if self.inch() { "lbf·ft" } else { "N·m" }
    }

    /// Force: N or lbf.
    pub fn force(self, n: f64) -> f64 {
        n * self.k(LBF_PER_N)
    }
    pub fn force_unit(self) -> &'static str {
        if self.inch() { "lbf" } else { "N" }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() <= 1e-6 * b.abs().max(1.0)
    }

    #[test]
    fn conversions() {
        let i = Units::Inch;
        assert!(close(i.len(25.4), 1.0));
        assert!(close(i.from_len(0.5), 12.7));
        assert!(close(i.speed(100.0), 328.083_99));
        assert!(close(i.from_speed(i.speed(123.0)), 123.0));
        assert!(close(i.mrr(16.387_064), 1.0));
        assert!(close(i.power(0.745_699_872), 1.0));
        assert!(close(i.force(4.448_221_615), 1.0));
        assert!(close(i.torque(1.355_817_948), 1.0));
        assert!(close(i.micro(25.4), 0.001));
        assert!(close(i.from_micro(0.001), 25.4));
        let m = Units::Metric;
        assert!(close(m.len(12.0), 12.0));
        assert!(close(m.speed(80.0), 80.0));
    }

    #[test]
    fn locale_default() {
        assert_eq!(Units::from_locale("en-US"), Units::Inch);
        assert_eq!(Units::from_locale("en_US.UTF-8"), Units::Inch);
        assert_eq!(Units::from_locale("en-GB"), Units::Metric);
        assert_eq!(Units::from_locale("it-IT"), Units::Metric);
        assert_eq!(Units::from_locale("zh-Hans-CN"), Units::Metric);
        assert_eq!(Units::from_locale("en"), Units::Metric);
    }
}
