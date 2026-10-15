//! Chinese, Japanese and Korean glyphs come from fonts already installed on the
//! system, loaded only when needed, so the executable stays small.

use eframe::egui::{self, FontData, FontDefinitions, FontFamily};
use std::path::PathBuf;
use std::sync::Arc;
use swarf::i18n::Lang;

const SCRIPTS: [Lang; 3] = [Lang::Zh, Lang::Ja, Lang::Ko];

#[derive(Default)]
pub struct CjkFonts {
    loaded: Vec<(Lang, Arc<FontData>)>,
    tried: bool,
    /// Han glyph style in use and whether CJK fonts lead the proportional family.
    applied: Option<(Lang, bool)>,
}

impl CjkFonts {
    /// Installs CJK fonts when `needed` (a CJK language is active, or the language
    /// menu may show CJK names) and keeps their order in line with `lang`.
    pub fn apply(&mut self, ctx: &egui::Context, lang: Lang, needed: bool) {
        if !needed && self.applied.is_none() {
            return;
        }
        // With a CJK language the system font leads, so Latin and CJK share one baseline.
        let cjk_first = lang.is_cjk();
        let primary = if cjk_first { lang } else { self.applied.map(|a| a.0).unwrap_or(Lang::Zh) };
        if self.applied == Some((primary, cjk_first)) {
            return;
        }
        if !self.tried {
            self.tried = true;
            for script in SCRIPTS {
                if let Some(data) = load_first(script) {
                    self.loaded.push((script, Arc::new(data)));
                }
            }
        }
        self.applied = Some((primary, cjk_first));
        if self.loaded.is_empty() {
            return;
        }
        let mut defs = FontDefinitions::default();
        let mut order: Vec<&(Lang, Arc<FontData>)> = self.loaded.iter().collect();
        order.sort_by_key(|(l, _)| *l != primary);
        let names: Vec<String> = order.iter().map(|(l, _)| format!("cjk-{l:?}")).collect();
        for (name, (_, data)) in names.iter().zip(&order) {
            defs.font_data.insert(name.clone(), data.clone());
        }
        let prop = defs.families.entry(FontFamily::Proportional).or_default();
        if cjk_first {
            prop.splice(0..0, names.iter().cloned());
        } else {
            prop.extend(names.iter().cloned());
        }
        defs.families.entry(FontFamily::Monospace).or_default().extend(names.iter().cloned());
        ctx.set_fonts(defs);
    }
}

fn load_first(lang: Lang) -> Option<FontData> {
    candidates(lang).into_iter().find_map(|(path, index)| {
        let bytes = std::fs::read(&path).ok()?;
        let mut data = FontData::from_owned(bytes);
        data.index = index;
        Some(data)
    })
}

/// Font files to try, best first, with the face index inside a collection.
fn candidates(lang: Lang) -> Vec<(PathBuf, u32)> {
    let mut out: Vec<(PathBuf, u32)> = Vec::new();
    if cfg!(target_os = "macos") {
        let dir = PathBuf::from("/System/Library/Fonts");
        let files: &[&str] = match lang {
            Lang::Ja => &["ヒラギノ角ゴシック W3.ttc", "Hiragino Sans GB.ttc"],
            Lang::Ko => &["AppleSDGothicNeo.ttc", "Supplemental/AppleGothic.ttf"],
            _ => &["Hiragino Sans GB.ttc", "STHeiti Medium.ttc"],
        };
        out.extend(files.iter().map(|f| (dir.join(f), 0)));
    } else if cfg!(target_os = "windows") {
        let dir = PathBuf::from(std::env::var_os("WINDIR").unwrap_or_else(|| "C:\\Windows".into())).join("Fonts");
        let files: &[&str] = match lang {
            Lang::Ja => &["YuGothM.ttc", "meiryo.ttc", "msgothic.ttc"],
            Lang::Ko => &["malgun.ttf", "gulim.ttc"],
            _ => &["msyh.ttc", "simhei.ttf", "simsun.ttc"],
        };
        out.extend(files.iter().map(|f| (dir.join(f), 0)));
    } else {
        // Noto Sans CJK collections hold JP, KR, SC, TC, HK in this order.
        let index = match lang {
            Lang::Ja => 0,
            Lang::Ko => 1,
            _ => 2,
        };
        for dir in ["/usr/share/fonts/opentype/noto", "/usr/share/fonts/noto-cjk", "/usr/share/fonts/google-noto-cjk", "/usr/share/fonts/OTF"] {
            out.push((PathBuf::from(dir).join("NotoSansCJK-Regular.ttc"), index));
        }
        let extra: &[&str] = match lang {
            Lang::Ja => &["/usr/share/fonts/opentype/ipafont-gothic/ipag.ttf"],
            Lang::Ko => &["/usr/share/fonts/truetype/nanum/NanumGothic.ttf"],
            _ => &["/usr/share/fonts/truetype/wqy/wqy-microhei.ttc", "/usr/share/fonts/wenquanyi/wqy-microhei/wqy-microhei.ttc"],
        };
        out.extend(extra.iter().map(|f| (PathBuf::from(f), 0)));
    }
    out
}
