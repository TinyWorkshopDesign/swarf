//! Translations. English is the reference language.
//!
//! Every key carries all languages in one array, so the compiler rejects a key
//! with a missing translation. Placeholders are `{0}`, `{1}`, … in order.

use crate::calc::{Limiter, Msg, Row, Step, Unit, Warning};
use crate::data::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Lang {
    En,
    It,
    De,
    Fr,
    Es,
    Zh,
    Ja,
    Ko,
}

pub const LANG_COUNT: usize = 8;

impl Lang {
    pub const ALL: [Lang; LANG_COUNT] = [
        Lang::En,
        Lang::It,
        Lang::De,
        Lang::Fr,
        Lang::Es,
        Lang::Zh,
        Lang::Ja,
        Lang::Ko,
    ];

    /// Language name written in that language.
    pub fn native_name(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::It => "Italiano",
            Lang::De => "Deutsch",
            Lang::Fr => "Français",
            Lang::Es => "Español",
            Lang::Zh => "简体中文",
            Lang::Ja => "日本語",
            Lang::Ko => "한국어",
        }
    }

    pub fn is_cjk(self) -> bool {
        matches!(self, Lang::Zh | Lang::Ja | Lang::Ko)
    }

    /// Picks a language from a BCP-47 tag such as "it-IT" or "zh-Hans-CN".
    pub fn from_locale(tag: &str) -> Lang {
        let primary = tag
            .split(['-', '_', '.'])
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        match primary.as_str() {
            "it" => Lang::It,
            "de" => Lang::De,
            "fr" => Lang::Fr,
            "es" => Lang::Es,
            "zh" => Lang::Zh,
            "ja" => Lang::Ja,
            "ko" => Lang::Ko,
            _ => Lang::En,
        }
    }

    fn idx(self) -> usize {
        self as usize
    }

    /// Thousands separator and decimal mark.
    fn separators(self) -> (&'static str, char) {
        match self {
            Lang::It | Lang::De | Lang::Es => (".", ','),
            Lang::Fr => (" ", ','),
            Lang::En | Lang::Zh | Lang::Ja | Lang::Ko => (",", '.'),
        }
    }

    pub fn t(self, k: K) -> &'static str {
        table(k)[self.idx()]
    }

    /// Translates a key and fills its `{0}`, `{1}`… placeholders.
    pub fn f(self, k: K, args: &[&str]) -> String {
        fill(self.t(k), args)
    }

    /// Formats a number with the language's separators.
    pub fn num(self, v: f64, dec: usize) -> String {
        if !v.is_finite() {
            return "—".into();
        }
        let (thousands, decimal) = self.separators();
        let s = format!("{:.*}", dec, v.abs());
        let (int, frac) = match s.split_once('.') {
            Some((i, f)) => (i.to_string(), Some(f.to_string())),
            None => (s.clone(), None),
        };
        let mut out = String::new();
        if v < 0.0 && s.chars().any(|c| c.is_ascii_digit() && c != '0') {
            out.push('-');
        }
        for (i, c) in int.chars().enumerate() {
            if i > 0 && (int.len() - i) % 3 == 0 {
                out.push_str(thousands);
            }
            out.push(c);
        }
        if let Some(f) = frac {
            out.push(decimal);
            out.push_str(&f);
        }
        out
    }
}

pub fn fill(template: &str, args: &[&str]) -> String {
    let mut out = template.to_string();
    for (i, a) in args.iter().enumerate() {
        out = out.replace(&format!("{{{i}}}"), a);
    }
    out
}

macro_rules! strings {
    ($($key:ident => [$($s:expr),* $(,)?]),* $(,)?) => {
        /// Translation keys.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum K { $($key),* }
        fn table(k: K) -> [&'static str; LANG_COUNT] {
            match k { $(K::$key => [$($s),*]),* }
        }
    };
}

// Column order: en, it, de, fr, es, zh, ja, ko
strings! {
    AppSubtitle => ["Cutting speeds and feeds for CNC milling and drilling", "Velocità e avanzamenti per fresatura e foratura CNC", "Schnittdaten für CNC-Fräsen und -Bohren", "Vitesses de coupe et avances pour le fraisage et le perçage CNC", "Velocidades de corte y avances para fresado y taladrado CNC", "CNC 铣削与钻削的切削速度和进给", "CNCフライス加工・穴あけの切削条件", "CNC 밀링·드릴링 절삭 속도와 이송"],
    LoadExample => ["Load example", "Esempio iniziale", "Beispiel laden", "Charger l'exemple", "Cargar ejemplo", "载入示例", "例を読み込む", "예제 불러오기"],
    LoadExampleTip => ["Restore the example values", "Ripristina i valori di esempio", "Beispielwerte wiederherstellen", "Rétablir les valeurs d'exemple", "Restaurar los valores de ejemplo", "恢复示例数值", "例の値に戻す", "예제 값으로 되돌리기"],
    Language => ["Language", "Lingua", "Sprache", "Langue", "Idioma", "语言", "言語", "언어"],

    SecMachine => ["Machine", "Macchina", "Maschine", "Machine", "Máquina", "机床", "機械", "기계"],
    SecMaterial => ["Material", "Materiale", "Werkstoff", "Matière", "Material", "材料", "被削材", "소재"],
    SecTool => ["Tool", "Utensile", "Werkzeug", "Outil", "Herramienta", "刀具", "工具", "공구"],
    SecCut => ["Cut", "Passata", "Schnitt", "Passe", "Pasada", "切削参数", "切込み", "절삭"],
    SecTweaks => ["Adjustments", "Regolazioni", "Feinabstimmung", "Réglages", "Ajustes", "调整", "調整", "조정"],
    SecLibrary => ["My setups", "Le mie impostazioni", "Meine Einstellungen", "Mes réglages", "Mis configuraciones", "我的设置", "保存した設定", "내 설정"],

    RpmMax => ["Max spindle speed", "Giri max", "Max. Drehzahl", "Vitesse broche max.", "Velocidad máx. husillo", "最高转速", "最高回転数", "최고 회전수"],
    RpmMin => ["Min spindle speed", "Giri min", "Min. Drehzahl", "Vitesse broche min.", "Velocidad mín. husillo", "最低转速", "最低回転数", "최저 회전수"],
    SpindlePower => ["Spindle power", "Potenza mandrino", "Spindelleistung", "Puissance broche", "Potencia del husillo", "主轴功率", "主軸出力", "주축 출력"],
    FullPowerFrom => ["Full power from", "Potenza piena da", "Volle Leistung ab", "Pleine puissance dès", "Potencia plena desde", "满功率起始转速", "定格出力の回転数", "정격 출력 회전수"],
    FullPowerTip => ["Below this speed power drops proportionally (constant torque)", "Sotto questi giri la potenza cala in proporzione (coppia costante)", "Unterhalb dieser Drehzahl sinkt die Leistung proportional (konstantes Drehmoment)", "Sous cette vitesse la puissance baisse proportionnellement (couple constant)", "Por debajo de esta velocidad la potencia baja en proporción (par constante)", "低于此转速时功率按比例下降（恒扭矩）", "この回転数以下では出力が比例して下がります（定トルク）", "이 회전수 아래에서는 출력이 비례해서 줄어듭니다(정토크)"],
    MaxFeed => ["Max cutting feed", "Avanz. max lavoro", "Max. Arbeitsvorschub", "Avance de travail max.", "Avance máx. de trabajo", "最大切削进给", "最大切削送り", "최대 절삭 이송"],
    Rigidity => ["Rigidity", "Rigidità", "Steifigkeit", "Rigidité", "Rigidez", "刚性", "剛性", "강성"],
    RigLow => ["Low (router, gantry)", "Bassa (router, portale)", "Niedrig (Portalfräse)", "Faible (routeur, portique)", "Baja (router, pórtico)", "低（雕刻机、龙门）", "低（ルーター、門型）", "낮음(라우터, 갠트리)"],
    RigMedium => ["Medium (benchtop, column)", "Media (banco, colonna)", "Mittel (Tisch-, Säulenfräse)", "Moyenne (d'établi, à colonne)", "Media (de banco, de columna)", "中（台式、立柱式）", "中（卓上、コラム型）", "보통(탁상형, 컬럼형)"],
    RigHigh => ["High (cast-iron VMC)", "Alta (VMC in ghisa)", "Hoch (BAZ in Guss)", "Élevée (centre d'usinage en fonte)", "Alta (centro de mecanizado de fundición)", "高（铸铁加工中心）", "高（鋳物マシニングセンタ）", "높음(주물 머시닝센터)"],
    PresetRouter => ["Hobby CNC router, 2.2 kW spindle", "Router CNC hobby, elettromandrino 2,2 kW", "Hobby-CNC-Fräse, 2,2-kW-Spindel", "Routeur CNC amateur, broche 2,2 kW", "Router CNC aficionado, husillo 2,2 kW", "业余 CNC 雕刻机，2.2 kW 主轴", "ホビー用CNCルーター、2.2 kW主軸", "취미용 CNC 라우터, 2.2 kW 주축"],
    PresetBench => ["Benchtop CNC mill, 1.1 kW", "Fresatrice da banco CNC, 1,1 kW", "CNC-Tischfräse, 1,1 kW", "Fraiseuse CNC d'établi, 1,1 kW", "Fresadora CNC de banco, 1,1 kW", "台式 CNC 铣床，1.1 kW", "卓上CNCフライス盤、1.1 kW", "탁상형 CNC 밀링, 1.1 kW"],
    PresetVmc => ["Vertical machining centre ISO40, 15 kW", "Centro di lavoro verticale ISO40, 15 kW", "Vertikal-BAZ ISO40, 15 kW", "Centre d'usinage vertical ISO40, 15 kW", "Centro de mecanizado vertical ISO40, 15 kW", "立式加工中心 ISO40，15 kW", "立形マシニングセンタ ISO40、15 kW", "수직형 머시닝센터 ISO40, 15 kW"],
    PresetCustom => ["Custom", "Personalizzata", "Benutzerdefiniert", "Personnalisée", "Personalizada", "自定义", "カスタム", "사용자 지정"],

    GroupLine => ["ISO group {0} · {1} · kc1.1 {2} N/mm²", "Gruppo ISO {0} · {1} · kc1.1 {2} N/mm²", "ISO-Gruppe {0} · {1} · kc1.1 {2} N/mm²", "Groupe ISO {0} · {1} · kc1.1 {2} N/mm²", "Grupo ISO {0} · {1} · kc1.1 {2} N/mm²", "ISO 组 {0} · {1} · kc1.1 {2} N/mm²", "ISOグループ {0} · {1} · kc1.1 {2} N/mm²", "ISO 그룹 {0} · {1} · kc1.1 {2} N/mm²"],
    IsoN => ["Non-ferrous", "Non ferrosi", "NE-Metalle", "Non ferreux", "No ferrosos", "有色金属", "非鉄金属", "비철금속"],
    IsoP => ["Steels", "Acciai", "Stähle", "Aciers", "Aceros", "钢", "鋼", "강"],
    IsoM => ["Stainless steels", "Inox", "Rostfreie Stähle", "Aciers inoxydables", "Aceros inoxidables", "不锈钢", "ステンレス鋼", "스테인리스강"],
    IsoK => ["Cast irons", "Ghise", "Gusseisen", "Fontes", "Fundiciones", "铸铁", "鋳鉄", "주철"],
    IsoS => ["Superalloys and titanium", "Superleghe e titanio", "Superlegierungen und Titan", "Superalliages et titane", "Superaleaciones y titanio", "高温合金和钛", "耐熱合金・チタン", "내열합금 및 티타늄"],
    IsoH => ["Hardened steels", "Acciai temprati", "Gehärtete Stähle", "Aciers trempés", "Aceros templados", "淬硬钢", "焼入れ鋼", "경화강"],
    IsoO => ["Plastics and composites", "Plastiche e compositi", "Kunststoffe und Verbundwerkstoffe", "Plastiques et composites", "Plásticos y compuestos", "塑料和复合材料", "樹脂・複合材", "플라스틱 및 복합재"],
    IsoW => ["Wood and wood products", "Legno e derivati", "Holz und Holzwerkstoffe", "Bois et dérivés", "Madera y derivados", "木材及木制品", "木材・木質材料", "목재 및 목질재"],

    ToolType => ["Type", "Tipo", "Typ", "Type", "Tipo", "类型", "種類", "종류"],
    ToolMat => ["Tool material", "Materiale utensile", "Schneidstoff", "Matériau de l'outil", "Material de la herramienta", "刀具材料", "工具材種", "공구 재질"],
    CoatingLabel => ["Coating", "Rivestimento", "Beschichtung", "Revêtement", "Recubrimiento", "涂层", "コーティング", "코팅"],
    Diameter => ["Diameter Ø", "Diametro Ø", "Durchmesser Ø", "Diamètre Ø", "Diámetro Ø", "直径 Ø", "直径 Ø", "직경 Ø"],
    Flutes => ["Flutes Z", "Taglienti Z", "Schneiden Z", "Dents Z", "Filos Z", "刃数 Z", "刃数 Z", "날 수 Z"],
    CornerR => ["Corner radius r", "Raggio r", "Eckenradius r", "Rayon de coin r", "Radio de esquina r", "圆角半径 r", "コーナ半径 r", "코너 반경 r"],
    VAngle => ["Included V angle", "Angolo V incluso", "Spitzenwinkel", "Angle de pointe", "Ángulo de la V", "V 形夹角", "V角（全角）", "V 각도"],
    TipD => ["Tip Ø", "Ø punta", "Spitzen-Ø", "Ø de pointe", "Ø de punta", "刀尖直径", "先端径", "팁 직경"],
    LeadAngle => ["Lead angle κ", "Angolo registro κ", "Einstellwinkel κ", "Angle de direction κ", "Ángulo de posición κ", "主偏角 κ", "切込み角 κ", "절입각 κ"],
    FluteLen => ["Flute length", "Lunghezza tagliente", "Schneidenlänge", "Longueur de coupe", "Longitud de corte", "刃长", "刃長", "날 길이"],
    Stickout => ["Stickout from spindle", "Sporgenza dal mandrino", "Auskraglänge", "Porte-à-faux", "Voladizo", "悬伸长度", "突き出し長さ", "돌출 길이"],
    ShankD => ["Shank Ø", "Ø gambo", "Schaft-Ø", "Ø de queue", "Ø de mango", "柄径", "シャンク径", "생크 직경"],
    KindFlat => ["Flat end mill", "Fresa cilindrica piana", "Schaftfräser", "Fraise deux tailles", "Fresa plana", "平底立铣刀", "スクエアエンドミル", "평 엔드밀"],
    KindBall => ["Ball end mill", "Fresa sferica", "Kugelfräser", "Fraise hémisphérique", "Fresa esférica", "球头铣刀", "ボールエンドミル", "볼 엔드밀"],
    KindBull => ["Bull-nose end mill", "Fresa torica", "Torusfräser", "Fraise torique", "Fresa tórica", "圆鼻刀", "ラジアスエンドミル", "코너R 엔드밀"],
    KindVBit => ["V-bit / chamfer mill", "Fresa a V / smusso", "V-Fräser / Fasenfräser", "Fraise en V / à chanfreiner", "Fresa en V / de chaflán", "V 形刀 / 倒角刀", "Vカッター / 面取りカッター", "V 비트 / 챔퍼 커터"],
    KindFace => ["Indexable face mill", "Spianatore a inserti", "Planfräser mit Wendeplatten", "Fraise à surfacer à plaquettes", "Planeadora de insertos", "可转位面铣刀", "正面フライス（刃先交換式）", "인서트 페이스 밀"],
    KindDrill => ["Twist drill", "Punta elicoidale", "Spiralbohrer", "Foret hélicoïdal", "Broca helicoidal", "麻花钻", "ツイストドリル", "트위스트 드릴"],
    MatCarbide => ["Carbide", "Metallo duro (MD)", "Hartmetall (HM)", "Carbure monobloc", "Metal duro", "硬质合金", "超硬", "초경"],
    MatCarbideShort => ["carbide", "MD", "HM", "carbure", "MD", "硬质合金", "超硬", "초경"],
    MatHssCo => ["HSS-Co (cobalt)", "HSS-Co (cobalto)", "HSS-Co (Kobalt)", "HSS-Co (cobalt)", "HSS-Co (cobalto)", "含钴高速钢", "コバルトハイス", "코발트 하이스"],
    MatHss => ["HSS", "HSS", "HSS", "HSS", "HSS", "高速钢", "ハイス", "하이스"],
    CoatNone => ["Uncoated", "Non rivestito", "Unbeschichtet", "Non revêtu", "Sin recubrimiento", "无涂层", "ノンコート", "무코팅"],
    CoatDiamond => ["CVD diamond", "Diamante CVD", "CVD-Diamant", "Diamant CVD", "Diamante CVD", "CVD 金刚石", "CVDダイヤモンド", "CVD 다이아몬드"],

    StratSlot => ["Full slot", "Cava piena", "Volle Nut", "Rainure pleine", "Ranura completa", "全槽铣", "溝加工", "풀 슬롯"],
    StratRough => ["Roughing", "Sgrossatura", "Schruppen", "Ébauche", "Desbaste", "粗加工", "荒加工", "황삭"],
    StratHsm => ["Trochoidal / adaptive", "Trocoidale / adattiva", "Trochoidal / adaptiv", "Trochoïdal / adaptatif", "Trocoidal / adaptativo", "摆线 / 自适应", "トロコイド / アダプティブ", "트로코이달 / 어댑티브"],
    StratFinish => ["Finishing", "Finitura", "Schlichten", "Finition", "Acabado", "精加工", "仕上げ", "정삭"],
    HoleDepth => ["Hole depth", "Profondità foro", "Bohrtiefe", "Profondeur de perçage", "Profundidad del agujero", "孔深", "穴深さ", "구멍 깊이"],
    AxialDepth => ["Axial depth ap", "Profondità assiale ap", "Schnitttiefe ap", "Profondeur axiale ap", "Profundidad axial ap", "轴向切深 ap", "軸方向切込み ap", "축방향 절입 ap"],
    AxialShort => ["Depth ap", "Profondità ap", "Tiefe ap", "Profondeur ap", "Profundidad ap", "切深 ap", "切込み ap", "절입 ap"],
    RadialWidth => ["Radial width ae", "Larghezza radiale ae", "Schnittbreite ae", "Engagement radial ae", "Ancho radial ae", "径向切宽 ae", "径方向切込み ae", "반경방향 절입 ae"],
    ChipThinning => ["Compensate chip thinning", "Compensa assottigliamento truciolo", "Spanausdünnung ausgleichen", "Compenser l'amincissement du copeau", "Compensar el adelgazamiento de viruta", "补偿切屑减薄", "切りくず薄化を補正", "칩 두께 감소 보정"],
    LimitDeflection => ["Limit deflection to", "Limita flessione a", "Durchbiegung begrenzen auf", "Limiter la flexion à", "Limitar la deflexión a", "挠度限制为", "たわみの上限", "처짐 한계"],
    LimitPower => ["Reduce feed when power is short", "Riduci l'avanzamento se la potenza non basta", "Vorschub senken, wenn die Leistung nicht reicht", "Réduire l'avance si la puissance manque", "Reducir el avance si falta potencia", "功率不足时降低进给", "出力不足時は送りを下げる", "출력이 부족하면 이송 감소"],

    ToolLife => ["Tool life", "Durata utensile", "Standzeit", "Durée de vie", "Vida útil", "刀具寿命", "工具寿命", "공구 수명"],
    Productivity => ["Productivity", "Produttività", "Produktivität", "Productivité", "Productividad", "效率", "生産性", "생산성"],
    FinishEnd => ["Finish", "Finitura", "Oberfläche", "Finition", "Acabado", "表面质量", "仕上げ面", "표면 품질"],
    RemovalEnd => ["Removal", "Asportazione", "Zerspanvolumen", "Enlèvement", "Arranque", "去除量", "除去量", "제거량"],

    NameHint => ["Name, e.g. 6 mm 3-flute aluminium", "Nome, es. Fresa 6 Z3 alluminio", "Name, z. B. 6-mm-Fräser Z3 Alu", "Nom, ex. fraise 6 mm Z3 alu", "Nombre, p. ej. fresa 6 mm Z3 aluminio", "名称，如 6 mm 三刃 铝", "名前（例：6 mm 3枚刃 アルミ）", "이름 (예: 6 mm 3날 알루미늄)"],
    Save => ["Save", "Salva", "Speichern", "Enregistrer", "Guardar", "保存", "保存", "저장"],
    LibEmpty => ["No saved setups yet. Save the current combination to load it later or compare it in the charts.", "Nessuna impostazione salvata. Salva la combinazione corrente per richiamarla o confrontarla nei grafici.", "Noch keine gespeicherten Einstellungen. Speichere die aktuelle Kombination, um sie später zu laden oder in den Diagrammen zu vergleichen.", "Aucun réglage enregistré. Enregistrez la combinaison actuelle pour la recharger ou la comparer dans les graphiques.", "Aún no hay configuraciones guardadas. Guarda la combinación actual para cargarla después o compararla en los gráficos.", "还没有保存的设置。保存当前组合，以便之后载入或在图表中比较。", "保存した設定はまだありません。現在の組み合わせを保存すると、後で読み込んだりグラフで比較したりできます。", "저장된 설정이 없습니다. 현재 조합을 저장하면 나중에 불러오거나 그래프에서 비교할 수 있습니다."],
    DefaultName => ["Setup {0}", "Impostazione {0}", "Einstellung {0}", "Réglage {0}", "Configuración {0}", "设置 {0}", "設定 {0}", "설정 {0}"],
    Load => ["Load", "Carica", "Laden", "Charger", "Cargar", "载入", "読込", "불러오기"],
    Delete => ["Delete", "Elimina", "Löschen", "Supprimer", "Eliminar", "删除", "削除", "삭제"],
    Confirm => ["Confirm", "Conferma", "Bestätigen", "Confirmer", "Confirmar", "确认", "確認", "확인"],
    CompareTip => ["Show in comparison", "Mostra nel confronto", "Im Vergleich zeigen", "Afficher dans la comparaison", "Mostrar en la comparación", "在比较中显示", "比較に表示", "비교에 표시"],

    CopyGcode => ["Copy G-code", "Copia G-code", "G-Code kopieren", "Copier le G-code", "Copiar código G", "复制 G 代码", "Gコードをコピー", "G코드 복사"],
    Copied => ["Copied", "Copiato", "Kopiert", "Copié", "Copiado", "已复制", "コピーしました", "복사됨"],
    UnitMmRev => ["mm/rev", "mm/giro", "mm/U", "mm/tr", "mm/rev", "mm/转", "mm/rev", "mm/rev"],
    SubDrill => ["Vc {0} m/min · f {1} {2}", "Vc {0} m/min · f {1} {2}", "Vc {0} m/min · f {1} {2}", "Vc {0} m/min · f {1} {2}", "Vc {0} m/min · f {1} {2}", "Vc {0} m/min · f {1} {2}", "Vc {0} m/min · f {1} {2}", "Vc {0} m/min · f {1} {2}"],
    SubMill => ["Vc {0} m/min · fz {1} mm · actual chip {2} mm", "Vc {0} m/min · fz {1} mm · truciolo reale {2} mm", "Vc {0} m/min · fz {1} mm · reale Spandicke {2} mm", "Vc {0} m/min · fz {1} mm · copeau réel {2} mm", "Vc {0} m/min · fz {1} mm · viruta real {2} mm", "Vc {0} m/min · fz {1} mm · 实际切屑厚度 {2} mm", "Vc {0} m/min · fz {1} mm · 実切りくず厚さ {2} mm", "Vc {0} m/min · fz {1} mm · 실제 칩 두께 {2} mm"],
    NoLimit => ["no limit", "nessun limite", "keine Begrenzung", "aucune limite", "sin límite", "无限制", "制限なし", "제한 없음"],
    LimitChip => ["limit: {0}", "limite: {0}", "Grenze: {0}", "limite : {0}", "límite: {0}", "限制：{0}", "制限：{0}", "제한: {0}"],
    LimRpmMax => ["max rpm", "giri max", "max. Drehzahl", "vitesse max.", "rpm máx.", "最高转速", "最高回転数", "최고 회전수"],
    LimRpmMin => ["min rpm", "giri min", "min. Drehzahl", "vitesse min.", "rpm mín.", "最低转速", "最低回転数", "최저 회전수"],
    LimFeedMax => ["max feed", "avanz. max", "max. Vorschub", "avance max.", "avance máx.", "最大进给", "最大送り", "최대 이송"],
    LimPower => ["power", "potenza", "Leistung", "puissance", "potencia", "功率", "出力", "출력"],
    LimDeflection => ["deflection", "flessione", "Durchbiegung", "flexion", "deflexión", "挠度", "たわみ", "처짐"],

    TabResults => ["Results", "Risultati", "Ergebnisse", "Résultats", "Resultados", "结果", "結果", "결과"],
    TabCharts => ["Comparison charts", "Grafici di confronto", "Vergleichsdiagramme", "Graphiques comparatifs", "Gráficos comparativos", "对比图表", "比較グラフ", "비교 그래프"],
    Details => ["Details", "Dettagli", "Details", "Détails", "Detalles", "详细", "詳細", "상세"],
    Warnings => ["Warnings", "Avvisi", "Hinweise", "Avertissements", "Avisos", "提示", "注意", "경고"],
    BalancedTitle => ["Balanced parameters.", "Parametri equilibrati.", "Ausgewogene Parameter.", "Paramètres équilibrés.", "Parámetros equilibrados.", "参数均衡。", "バランスの取れた条件です。", "균형 잡힌 조건입니다."],
    BalancedText => ["Start with the override at 100% and listen to the cut.", "Parti con l'override al 100% e ascolta il taglio.", "Mit Override 100 % beginnen und auf den Schnitt hören.", "Commencez avec l'override à 100 % et écoutez la coupe.", "Empieza con el override al 100 % y escucha el corte.", "以 100% 倍率开始，并留意切削声音。", "オーバーライド100%で始め、切削音を確認してください。", "오버라이드 100%로 시작하고 절삭음을 들어 보세요."],
    WhyNumbers => ["Where the numbers come from", "Da dove vengono i numeri", "Woher die Zahlen kommen", "D'où viennent les chiffres", "De dónde salen los números", "数值的由来", "数値の根拠", "수치의 근거"],
    Disclaimer => ["Values are a starting point computed from typical tables and standard formulas. Check them against the tool maker's catalogue and start conservatively.", "I valori sono un punto di partenza calcolato da dati tabellari tipici e formule standard. Verifica con il catalogo del produttore dell'utensile e parti prudente.", "Die Werte sind ein Ausgangspunkt aus typischen Tabellen und Standardformeln. Mit dem Katalog des Werkzeugherstellers abgleichen und vorsichtig beginnen.", "Ces valeurs sont un point de départ calculé à partir de tables usuelles et de formules standard. Vérifiez-les avec le catalogue du fabricant et commencez prudemment.", "Los valores son un punto de partida calculado con tablas típicas y fórmulas estándar. Compruébalos con el catálogo del fabricante y empieza con prudencia.", "数值是根据常用表格和标准公式计算的起点。请对照刀具厂商的样本核实，并从保守值开始。", "値は一般的な表と標準式から求めた目安です。工具メーカーのカタログで確認し、控えめな条件から始めてください。", "값은 일반적인 표와 표준 공식으로 계산한 출발점입니다. 공구 제조사 카탈로그로 확인하고 보수적으로 시작하세요."],

    FeedPerRev => ["Feed per revolution", "Avanzamento al giro", "Vorschub pro Umdrehung", "Avance par tour", "Avance por vuelta", "每转进给", "1回転あたり送り", "회전당 이송"],
    TargetX => ["target {0}", "obiettivo {0}", "Ziel {0}", "cible {0}", "objetivo {0}", "目标 {0}", "目標 {0}", "목표 {0}"],
    Mrr => ["Material removal rate", "Asportazione", "Zeitspanvolumen", "Débit de copeaux", "Tasa de arranque", "材料去除率", "切削体積", "소재 제거율"],
    AvailKw => ["available {0} kW", "disponibile {0} kW", "verfügbar {0} kW", "disponible {0} kW", "disponible {0} kW", "可用 {0} kW", "使用可能 {0} kW", "사용 가능 {0} kW"],
    AvailKwAt => ["available {0} kW at {1} rpm", "disponibile {0} kW a {1} giri", "verfügbar {0} kW bei {1} 1/min", "disponible {0} kW à {1} tr/min", "disponible {0} kW a {1} rpm", "{1} rpm 时可用 {0} kW", "{1} rpmで使用可能 {0} kW", "{1} rpm에서 사용 가능 {0} kW"],
    Torque => ["Torque", "Coppia", "Drehmoment", "Couple", "Par", "扭矩", "トルク", "토크"],
    Thrust => ["Axial thrust", "Spinta assiale", "Vorschubkraft", "Effort axial", "Empuje axial", "轴向推力", "スラスト力", "추력"],
    HoleTime => ["Time per hole", "Tempo per foro", "Zeit pro Bohrung", "Temps par trou", "Tiempo por agujero", "单孔时间", "1穴あたり時間", "구멍당 시간"],
    FeedOnly => ["feed only, {0} mm", "solo avanzamento, {0} mm", "nur Vorschub, {0} mm", "avance seule, {0} mm", "solo avance, {0} mm", "仅进给，{0} mm", "送りのみ、{0} mm", "이송만, {0} mm"],
    ActualChip => ["Actual chip thickness", "Spessore truciolo reale", "Tatsächliche Spandicke", "Épaisseur de copeau réelle", "Espesor de viruta real", "实际切屑厚度", "実切りくず厚さ", "실제 칩 두께"],
    ProgFz => ["programmed fz {0}", "fz programmato {0}", "programmiertes fz {0}", "fz programmé {0}", "fz programado {0}", "编程 fz {0}", "指令 fz {0}", "지령 fz {0}"],
    ThinningX => [" (×{0} thinning)", " (×{0} assottigl.)", " (×{0} Ausdünnung)", " (×{0} amincissement)", " (×{0} adelgazamiento)", "（×{0} 减薄）", "（×{0} 薄化）", " (×{0} 감소)"],
    ForceKc => ["cutting force {0} N · kc {1} N/mm²", "forza di taglio {0} N · kc {1} N/mm²", "Schnittkraft {0} N · kc {1} N/mm²", "effort de coupe {0} N · kc {1} N/mm²", "fuerza de corte {0} N · kc {1} N/mm²", "切削力 {0} N · kc {1} N/mm²", "切削抵抗 {0} N · kc {1} N/mm²", "절삭력 {0} N · kc {1} N/mm²"],
    TipDeflection => ["Tip deflection", "Flessione in punta", "Durchbiegung an der Spitze", "Flexion en bout", "Deflexión en la punta", "刀尖挠度", "先端たわみ", "선단 처짐"],
    DeflSub => ["limit {0} µm · stickout {1} mm", "limite {0} µm · sporgenza {1} mm", "Grenze {0} µm · Auskraglänge {1} mm", "limite {0} µm · porte-à-faux {1} mm", "límite {0} µm · voladizo {1} mm", "限值 {0} µm · 悬伸 {1} mm", "上限 {0} µm · 突き出し {1} mm", "한계 {0} µm · 돌출 {1} mm"],
    RampPlunge => ["Ramp and plunge", "Rampa e tuffo", "Rampe und Eintauchen", "Rampe et plongée", "Rampa y penetración", "斜坡与垂直下刀", "ランプと突っ込み", "램핑과 플런지"],
    RampSub => ["plunge F{0} · helix Ø {1}–{2} mm", "tuffo F{0} · elica Ø {1}–{2} mm", "Eintauchen F{0} · Helix Ø {1}–{2} mm", "plongée F{0} · hélice Ø {1}–{2} mm", "penetración F{0} · hélice Ø {1}–{2} mm", "垂直下刀 F{0} · 螺旋 Ø {1}–{2} mm", "突っ込み F{0} · ヘリカル Ø {1}–{2} mm", "플런지 F{0} · 헬리컬 Ø {1}–{2} mm"],
    Entry => ["Entry", "Entrata", "Einfahren", "Entrée", "Entrada", "切入", "進入", "진입"],
    EntryValue => ["from air", "in aria", "von außen", "hors matière", "desde fuera", "从外部切入", "材料外から", "소재 밖에서"],
    EntrySub => ["enter from the side on an arc, never plunge", "entra di lato con un arco, mai in tuffo", "seitlich im Bogen einfahren, nie eintauchen", "entrer par le côté en arc, jamais en plongée", "entra por el lateral en arco, nunca en penetración", "从侧面圆弧切入，切勿垂直下刀", "側面から円弧で進入し、突っ込まない", "측면에서 원호로 진입, 플런지 금지"],
    Scallop => ["Scallop between passes", "Cresta tra passate", "Rautiefe zwischen Bahnen", "Hauteur de crête entre passes", "Altura de cresta entre pasadas", "行间残留高度", "パス間のカスプ高さ", "패스 간 커스프 높이"],
    ScallopSub => ["with stepover ae {0} mm", "con passo ae {0} mm", "bei Zustellung ae {0} mm", "avec pas ae {0} mm", "con paso ae {0} mm", "行距 ae {0} mm", "ピック ae {0} mm", "스텝오버 ae {0} mm"],
    FeedCusp => ["Feed cusp (wall)", "Cresta da avanzamento (fianco)", "Vorschubrauheit (Wand)", "Crête d'avance (flanc)", "Cresta de avance (pared)", "进给残留（侧壁）", "送りマーク（側面）", "이송 자국(측벽)"],
    RaSub => ["theoretical Ra ≈ {0} µm", "Ra teorica ≈ {0} µm", "theoretisches Ra ≈ {0} µm", "Ra théorique ≈ {0} µm", "Ra teórica ≈ {0} µm", "理论 Ra ≈ {0} µm", "理論 Ra ≈ {0} µm", "이론 Ra ≈ {0} µm"],

    Curve => ["Curve", "Curva", "Kurve", "Courbe", "Curva", "曲线", "曲線", "곡선"],
    Vary => ["Vary:", "Fai variare:", "Variieren:", "Faire varier :", "Variar:", "变量：", "変化させる：", "변수:"],
    CurveNote => ["Everything else stays fixed. Marked points are where power, deflection or max feed hold the cut back.", "Tutto il resto resta fisso. I tratti marcati sono quelli in cui potenza, flessione o avanzamento massimo frenano il taglio.", "Alles andere bleibt gleich. Markierte Punkte zeigen, wo Leistung, Durchbiegung oder max. Vorschub den Schnitt bremsen.", "Tout le reste est fixe. Les points marqués indiquent où la puissance, la flexion ou l'avance max. freinent la coupe.", "Todo lo demás queda fijo. Los puntos marcados indican dónde la potencia, la deflexión o el avance máximo frenan el corte.", "其余参数保持不变。标记点表示功率、挠度或最大进给限制了切削。", "他の条件は固定です。マーク付きの点は、出力・たわみ・最大送りで切削が抑えられている所です。", "나머지는 고정됩니다. 표시된 점은 출력, 처짐 또는 최대 이송이 절삭을 제한하는 구간입니다."],
    XRadial => ["ae as % of Ø", "ae in % del Ø", "ae in % von Ø", "ae en % de Ø", "ae en % del Ø", "ae 占 Ø 的 %", "ae（Øに対する%）", "ae (Ø 대비 %)"],
    MrrTitle => ["Material removal rate MRR (cm³/min)", "Asportazione MRR (cm³/min)", "Zeitspanvolumen Q (cm³/min)", "Débit de copeaux Q (cm³/min)", "Tasa de arranque MRR (cm³/min)", "材料去除率 MRR（cm³/min）", "切削体積 MRR（cm³/min）", "소재 제거율 MRR (cm³/min)"],
    PowerTitle => ["Spindle power (kW)", "Potenza al mandrino (kW)", "Spindelleistung (kW)", "Puissance broche (kW)", "Potencia del husillo (kW)", "主轴功率（kW）", "主軸出力（kW）", "주축 출력 (kW)"],
    LegLimited => ["held back by a limit", "frenato da un limite", "durch Grenze gebremst", "freiné par une limite", "frenado por un límite", "受限制", "制限あり", "제한됨"],
    LegCurrent => ["current", "attuale", "aktuell", "actuel", "actual", "当前", "現在", "현재"],
    LegAvailable => ["available", "disponibile", "verfügbar", "disponible", "disponible", "可用", "使用可能", "사용 가능"],
    LegRequired => ["required", "richiesta", "benötigt", "requise", "requerida", "所需", "必要", "필요"],
    Compare => ["Comparison", "Confronto", "Vergleich", "Comparaison", "Comparación", "比较", "比較", "비교"],
    CompareNote => ["The four strategies applied to the current tool, plus saved setups with \"compare\" ticked.", "Le quattro strategie applicate all'utensile attuale, più le impostazioni salvate con la spunta \"confronta\".", "Die vier Strategien mit dem aktuellen Werkzeug, dazu gespeicherte Einstellungen mit Vergleichs-Haken.", "Les quatre stratégies appliquées à l'outil actuel, plus les réglages enregistrés cochés pour la comparaison.", "Las cuatro estrategias aplicadas a la herramienta actual, más las configuraciones guardadas marcadas para comparar.", "当前刀具的四种策略，以及勾选了比较的已保存设置。", "現在の工具に4つの戦略を当てはめた結果と、比較にチェックした保存済み設定。", "현재 공구에 네 가지 전략을 적용한 결과와 비교에 체크한 저장된 설정."],
    CurrentCut => ["Current cut", "Passata attuale", "Aktueller Schnitt", "Passe actuelle", "Pasada actual", "当前切削", "現在の条件", "현재 절삭"],
    BarPower => ["Spindle power kW (tick = available)", "Potenza mandrino kW (tacca = disponibile)", "Spindelleistung kW (Strich = verfügbar)", "Puissance broche kW (repère = disponible)", "Potencia del husillo kW (marca = disponible)", "主轴功率 kW（刻线 = 可用）", "主軸出力 kW（目盛 = 使用可能）", "주축 출력 kW (눈금 = 사용 가능)"],

    StepVcBase => ["Base Vc from table ({0})", "Vc base da tabella ({0})", "Basis-Vc aus Tabelle ({0})", "Vc de base (table, {0})", "Vc base de tabla ({0})", "表格基准 Vc（{0}）", "表の基準Vc（{0}）", "표 기준 Vc ({0})"],
    StepDrilling => ["Drilling", "Foratura", "Bohren", "Perçage", "Taladrado", "钻削", "穴あけ", "드릴링"],
    StepCoating => ["Coating {0}", "Rivestimento {0}", "Beschichtung {0}", "Revêtement {0}", "Recubrimiento {0}", "涂层 {0}", "コーティング {0}", "코팅 {0}"],
    StepFullSlot => ["Full slot (more heat)", "Cava piena (più calore)", "Volle Nut (mehr Wärme)", "Rainure pleine (plus de chaleur)", "Ranura completa (más calor)", "全槽（发热更多）", "溝加工（発熱大）", "풀 슬롯(발열 큼)"],
    StepEngCool => ["Radial engagement {0}% (less heat)", "Impegno radiale {0}% (meno calore)", "Radiale Zustellung {0} % (weniger Wärme)", "Engagement radial {0} % (moins de chaleur)", "Ancho radial {0} % (menos calor)", "径向切宽 {0}%（发热更少）", "径方向切込み {0}%（発熱小）", "반경방향 절입 {0}% (발열 적음)"],
    StepEng => ["Radial engagement {0}%", "Impegno radiale {0}%", "Radiale Zustellung {0} %", "Engagement radial {0} %", "Ancho radial {0} %", "径向切宽 {0}%", "径方向切込み {0}%", "반경방향 절입 {0}%"],
    StepLife => ["Life / productivity adjustment", "Regolazione durata/produttività", "Einstellung Standzeit/Produktivität", "Réglage durée de vie/productivité", "Ajuste vida/productividad", "寿命/效率调整", "寿命・生産性の調整", "수명/생산성 조정"],
    StepVcTarget => ["Target Vc", "Vc obiettivo", "Ziel-Vc", "Vc cible", "Vc objetivo", "目标 Vc", "目標Vc", "목표 Vc"],
    StepEffD => ["Effective Ø at depth ap", "Ø efficace alla profondità ap", "Effektiver Ø bei Tiefe ap", "Ø effectif à la profondeur ap", "Ø efectivo a la profundidad ap", "切深 ap 处的有效直径", "切込み ap での有効径", "절입 ap에서의 유효 직경"],
    StepFzBase => ["Base feed per tooth {0} × Ø", "Carico per dente base {0} × Ø", "Basis-Zahnvorschub {0} × Ø", "Avance par dent de base {0} × Ø", "Avance por diente base {0} × Ø", "基准每齿进给 {0} × Ø", "基準1刃送り {0} × Ø", "기준 날당 이송 {0} × Ø"],
    StepFzInsert => ["Base feed per tooth (insert)", "Carico per dente base (inserto)", "Basis-Zahnvorschub (Wendeplatte)", "Avance par dent de base (plaquette)", "Avance por diente base (inserto)", "基准每齿进给（刀片）", "基準1刃送り（インサート）", "기준 날당 이송(인서트)"],
    StepFzFactors => ["Tool ×{0} · rigidity ×{1} · adjustment ×{2}", "Utensile ×{0} · rigidità ×{1} · regolazione ×{2}", "Werkzeug ×{0} · Steifigkeit ×{1} · Einstellung ×{2}", "Outil ×{0} · rigidité ×{1} · réglage ×{2}", "Herramienta ×{0} · rigidez ×{1} · ajuste ×{2}", "刀具 ×{0} · 刚性 ×{1} · 调整 ×{2}", "工具 ×{0} · 剛性 ×{1} · 調整 ×{2}", "공구 ×{0} · 강성 ×{1} · 조정 ×{2}"],
    StepSlotSuffix => [" · slot ×{0}", " · cava ×{0}", " · Nut ×{0}", " · rainure ×{0}", " · ranura ×{0}", " · 全槽 ×{0}", " · 溝 ×{0}", " · 슬롯 ×{0}"],
    StepChipTarget => ["Target chip thickness hex", "Spessore truciolo obiettivo hex", "Ziel-Spandicke hex", "Épaisseur de copeau cible hex", "Espesor de viruta objetivo hex", "目标切屑厚度 hex", "目標切りくず厚さ hex", "목표 칩 두께 hex"],
    StepRadialThin => ["Radial thinning (ae < Ø/2)", "Assottigliamento radiale (ae < Ø/2)", "Radiale Ausdünnung (ae < Ø/2)", "Amincissement radial (ae < Ø/2)", "Adelgazamiento radial (ae < Ø/2)", "径向减薄（ae < Ø/2）", "径方向の薄化（ae < Ø/2）", "반경방향 감소 (ae < Ø/2)"],
    StepBallThin => ["Ball-nose axial thinning", "Assottigliamento assiale sfera", "Axiale Ausdünnung Kugel", "Amincissement axial (bout hémisphérique)", "Adelgazamiento axial de la esférica", "球头轴向减薄", "ボールの軸方向薄化", "볼 축방향 감소"],
    StepLeadThin => ["Lead angle κ {0}°", "Angolo di registro κ {0}°", "Einstellwinkel κ {0}°", "Angle de direction κ {0}°", "Ángulo de posición κ {0}°", "主偏角 κ {0}°", "切込み角 κ {0}°", "절입각 κ {0}°"],
    StepCapped => ["Capped at ×5 for safety", "Limitato a ×5 per sicurezza", "Zur Sicherheit auf ×5 begrenzt", "Plafonné à ×5 par sécurité", "Limitado a ×5 por seguridad", "为安全限制在 ×5", "安全のため×5に制限", "안전을 위해 ×5로 제한"],
    StepReduction => ["Reduction for {0}", "Riduzione per {0}", "Reduzierung wegen {0}", "Réduction pour {0}", "Reducción por {0}", "因{0}降低", "{0}による低減", "{0}로 인한 감소"],
    StepFzProg => ["Programmed fz", "fz programmato", "Programmiertes fz", "fz programmé", "fz programado", "编程 fz", "指令 fz", "지령 fz"],
    StepFnBase => ["Base feed per rev {0} × Ø", "Avanzamento al giro base {0} × Ø", "Basis-Vorschub pro Umdrehung {0} × Ø", "Avance par tour de base {0} × Ø", "Avance por vuelta base {0} × Ø", "基准每转进给 {0} × Ø", "基準回転送り {0} × Ø", "기준 회전당 이송 {0} × Ø"],
    StepFnFactors => ["Tool · rigidity · adjustment", "Utensile · rigidità · regolazione", "Werkzeug · Steifigkeit · Einstellung", "Outil · rigidité · réglage", "Herramienta · rigidez · ajuste", "刀具 · 刚性 · 调整", "工具 · 剛性 · 調整", "공구 · 강성 · 조정"],
    StepFnTarget => ["Target f", "f obiettivo", "Ziel-f", "f cible", "f objetivo", "目标 f", "目標 f", "목표 f"],

    WHssT => ["HSS not suitable", "HSS non adatto", "HSS ungeeignet", "HSS inadapté", "HSS no adecuado", "不适合高速钢", "ハイスは不適", "하이스 부적합"],
    WHssX => ["{0} can only be cut with carbide.", "{0} si lavora solo con metallo duro.", "{0} lässt sich nur mit Hartmetall bearbeiten.", "{0} ne s'usine qu'au carbure.", "{0} solo se mecaniza con metal duro.", "{0} 只能用硬质合金加工。", "{0}は超硬工具でしか加工できません。", "{0}은(는) 초경 공구로만 가공할 수 있습니다."],
    WAltinT => ["AlTiN on aluminium", "AlTiN su alluminio", "AlTiN auf Aluminium", "AlTiN sur aluminium", "AlTiN en aluminio", "铝件使用 AlTiN 涂层", "アルミにAlTiN", "알루미늄에 AlTiN"],
    WAltinX => ["The coating tends to build up. ZrN/DLC or polished uncoated is better.", "Il rivestimento tende a impastarsi. Meglio ZrN/DLC o non rivestito lucido.", "Die Beschichtung neigt zu Aufbauschneiden. Besser ZrN/DLC oder unbeschichtet poliert.", "Le revêtement tend à former une arête rapportée. Préférez ZrN/DLC ou non revêtu poli.", "El recubrimiento tiende a empastarse. Mejor ZrN/DLC o sin recubrir pulido.", "该涂层容易粘刀，建议使用 ZrN/DLC 或抛光无涂层刀具。", "溶着しやすいコーティングです。ZrN/DLCか鏡面ノンコートが適しています。", "코팅에 용착이 생기기 쉽습니다. ZrN/DLC 또는 연마된 무코팅이 좋습니다."],
    WDiaT => ["Diamond does not suit ferrous metals", "Il diamante non va sui ferrosi", "Diamant ist für Eisenwerkstoffe ungeeignet", "Le diamant ne convient pas aux ferreux", "El diamante no sirve para ferrosos", "金刚石不适合黑色金属", "ダイヤモンドは鉄系に不適", "다이아몬드는 철계 소재에 부적합"],
    WDiaX => ["Carbon diffuses into iron and the edge wears out within a few metres.", "Il carbonio diffonde nel ferro e il tagliente si consuma in pochi metri.", "Kohlenstoff diffundiert ins Eisen, die Schneide verschleißt nach wenigen Metern.", "Le carbone diffuse dans le fer et l'arête s'use en quelques mètres.", "El carbono se difunde en el hierro y el filo se gasta en pocos metros.", "碳会扩散到铁中，切削刃几米内就会磨损。", "炭素が鉄に拡散し、刃先は数メートルで摩耗します。", "탄소가 철로 확산되어 날이 몇 미터 만에 마모됩니다."],
    WPowT => ["Not enough power", "Potenza insufficiente", "Leistung reicht nicht", "Puissance insuffisante", "Potencia insuficiente", "功率不足", "出力不足", "출력 부족"],
    WPowX => ["Needs {0} kW at the spindle, {1} kW available at {2} rpm.", "Servono {0} kW al mandrino, disponibili {1} kW a {2} giri.", "Benötigt {0} kW an der Spindel, verfügbar {1} kW bei {2} 1/min.", "Il faut {0} kW à la broche, {1} kW disponibles à {2} tr/min.", "Se necesitan {0} kW en el husillo, hay {1} kW a {2} rpm.", "主轴需要 {0} kW，{2} rpm 时仅有 {1} kW。", "主軸に {0} kW 必要ですが、{2} rpm で使用可能なのは {1} kW です。", "주축에 {0} kW가 필요하지만 {2} rpm에서 {1} kW만 사용 가능합니다."],
    WDeepHoleT => ["Deep hole ({0}×Ø)", "Foro profondo ({0}×Ø)", "Tiefe Bohrung ({0}×Ø)", "Trou profond ({0}×Ø)", "Agujero profundo ({0}×Ø)", "深孔（{0}×Ø）", "深穴（{0}×Ø）", "깊은 구멍 ({0}×Ø)"],
    WDeepHoleX => ["Use the G83 cycle with chip clearing every {0} mm.", "Usa il ciclo G83 con scarico truciolo ogni {0} mm.", "G83-Zyklus mit Entspanen alle {0} mm verwenden.", "Utilisez le cycle G83 avec débourrage tous les {0} mm.", "Usa el ciclo G83 con descarga de viruta cada {0} mm.", "使用 G83 循环，每 {0} mm 退刀排屑。", "G83サイクルで {0} mm ごとに切りくずを排出してください。", "G83 사이클로 {0} mm마다 칩을 배출하세요."],
    WOver8T => ["Deeper than 8×Ø", "Oltre 8×Ø", "Tiefer als 8×Ø", "Plus de 8×Ø", "Más de 8×Ø", "超过 8×Ø", "8×Ø超", "8×Ø 초과"],
    WOver8X => ["Drill a short pilot hole with a stiff drill first, and cut feed by 20–30% at depth.", "Fai prima un foro pilota corto con una punta rigida e riduci l'avanzamento del 20–30% in profondità.", "Zuerst kurz mit einem steifen Bohrer pilotbohren und in der Tiefe den Vorschub um 20–30 % senken.", "Faites d'abord un avant-trou court avec un foret rigide et réduisez l'avance de 20 à 30 % en profondeur.", "Haz primero un agujero piloto corto con una broca rígida y reduce el avance un 20–30 % en profundidad.", "先用刚性好的钻头钻短引导孔，深处进给降低 20–30%。", "まず剛性の高いドリルで短い下穴をあけ、深い所では送りを20〜30%下げてください。", "먼저 강성 높은 드릴로 짧은 파일럿 구멍을 내고, 깊은 곳에서는 이송을 20–30% 줄이세요."],
    WVbitT => ["Deeper than the cone", "Profondità oltre il cono", "Tiefer als der Kegel", "Plus profond que le cône", "Más profundo que el cono", "超出锥形部分", "円錐部を超える深さ", "원뿔부보다 깊음"],
    WVbitX => ["At {0} mm the V is wider than the tool diameter.", "A {0} mm la V supera il Ø dell'utensile.", "Bei {0} mm ist das V breiter als der Werkzeug-Ø.", "À {0} mm, le V dépasse le Ø de l'outil.", "A {0} mm la V supera el Ø de la herramienta.", "在 {0} mm 深度时 V 形宽度超过刀具直径。", "{0} mm ではV幅が工具径を超えます。", "{0} mm에서 V 폭이 공구 직경을 넘습니다."],
    WThinT => ["Chip too thin ({0} mm)", "Truciolo troppo sottile ({0} mm)", "Span zu dünn ({0} mm)", "Copeau trop fin ({0} mm)", "Viruta demasiado fina ({0} mm)", "切屑过薄（{0} mm）", "切りくずが薄すぎます（{0} mm）", "칩이 너무 얇음 ({0} mm)"],
    WThinMetalX => ["The tool rubs and work-hardens the material instead of cutting. Raise the feed or lower the speed.", "L'utensile sfrega e incrudisce invece di tagliare. Aumenta l'avanzamento o riduci i giri.", "Das Werkzeug reibt und verfestigt den Werkstoff, statt zu schneiden. Vorschub erhöhen oder Drehzahl senken.", "L'outil frotte et écrouit la matière au lieu de couper. Augmentez l'avance ou baissez la vitesse.", "La herramienta roza y endurece el material en lugar de cortar. Sube el avance o baja la velocidad.", "刀具在摩擦并使材料加工硬化，而不是切削。请提高进给或降低转速。", "工具が削らずにこすれ、加工硬化を起こします。送りを上げるか回転数を下げてください。", "공구가 절삭하지 못하고 문질러 가공경화를 일으킵니다. 이송을 높이거나 회전수를 낮추세요."],
    WThinPlasticX => ["The plastic melts and sticks to the edge. Raise the feed or lower the speed.", "La plastica fonde e si impasta sul tagliente. Aumenta l'avanzamento o riduci i giri.", "Der Kunststoff schmilzt und klebt an der Schneide. Vorschub erhöhen oder Drehzahl senken.", "Le plastique fond et colle à l'arête. Augmentez l'avance ou baissez la vitesse.", "El plástico se funde y se pega al filo. Sube el avance o baja la velocidad.", "塑料熔化并粘在刃口上。请提高进给或降低转速。", "樹脂が溶けて刃先に付着します。送りを上げるか回転数を下げてください。", "플라스틱이 녹아 날에 달라붙습니다. 이송을 높이거나 회전수를 낮추세요."],
    WDeflHalfT => ["Deflection halves the feed", "La flessione dimezza l'avanzamento", "Durchbiegung halbiert den Vorschub", "La flexion divise l'avance par deux", "La deflexión reduce el avance a la mitad", "挠度使进给减半", "たわみで送りが半減", "처짐으로 이송이 절반으로 감소"],
    WDeflHalfX => ["Shorten the stickout, reduce ap/ae or use a larger tool.", "Accorcia la sporgenza, riduci ap/ae o usa un utensile più grosso.", "Auskraglänge verkürzen, ap/ae verringern oder ein größeres Werkzeug nehmen.", "Réduisez le porte-à-faux, diminuez ap/ae ou prenez un outil plus gros.", "Acorta el voladizo, reduce ap/ae o usa una herramienta más grande.", "缩短悬伸、减小 ap/ae 或换用更大的刀具。", "突き出しを短くするか、ap/aeを減らすか、太い工具を使ってください。", "돌출을 줄이거나 ap/ae를 줄이거나 더 큰 공구를 쓰세요."],
    WPowHalfT => ["Power at the limit", "Potenza al limite", "Leistung am Limit", "Puissance à la limite", "Potencia al límite", "功率已达上限", "出力が限界", "출력 한계"],
    WPowHalfX => ["Better to reduce ap or ae and keep a thicker chip.", "Conviene ridurre ap o ae e tenere un truciolo più spesso.", "Besser ap oder ae verringern und die Spandicke halten.", "Mieux vaut réduire ap ou ae et garder un copeau plus épais.", "Conviene reducir ap o ae y mantener una viruta más gruesa.", "建议减小 ap 或 ae，保持较厚的切屑。", "apかaeを減らし、切りくずを厚めに保つ方が有利です。", "ap 또는 ae를 줄이고 칩을 두껍게 유지하는 것이 좋습니다."],
    WDeflT => ["Deflection {0} µm", "Flessione {0} µm", "Durchbiegung {0} µm", "Flexion {0} µm", "Deflexión {0} µm", "挠度 {0} µm", "たわみ {0} µm", "처짐 {0} µm"],
    WDeflX => ["Above the {0} µm limit: expect chatter and parts out of tolerance.", "Oltre il limite di {0} µm: aspettati vibrazioni e quote fuori tolleranza.", "Über der Grenze von {0} µm: Rattern und Maße außer Toleranz zu erwarten.", "Au-delà de la limite de {0} µm : vibrations et cotes hors tolérance probables.", "Supera el límite de {0} µm: espera vibraciones y cotas fuera de tolerancia.", "超过 {0} µm 限值：可能出现振刀和尺寸超差。", "上限 {0} µm 超過：びびりや寸法外れが起こりやすくなります。", "한계 {0} µm 초과: 채터와 공차 이탈이 예상됩니다."],
    WApFluteT => ["ap beyond the flute length", "ap oltre la lunghezza tagliente", "ap größer als die Schneidenlänge", "ap au-delà de la longueur de coupe", "ap mayor que la longitud de corte", "ap 超过刃长", "apが刃長を超えています", "ap가 날 길이를 초과"],
    WApFluteX => ["With {0} mm of flute the shank would rub on the part.", "Con {0} mm di tagliente il gambo strisciarebbe sul pezzo.", "Bei {0} mm Schneidenlänge reibt der Schaft am Werkstück.", "Avec {0} mm de coupe, la queue frotterait sur la pièce.", "Con {0} mm de corte el mango rozaría la pieza.", "刃长只有 {0} mm，刀柄会摩擦工件。", "刃長 {0} mm ではシャンクがワークに擦れます。", "날 길이가 {0} mm라 생크가 공작물에 닿습니다."],
    WStickT => ["Stickout {0}×Ø", "Sporgenza {0}×Ø", "Auskraglänge {0}×Ø", "Porte-à-faux {0}×Ø", "Voladizo {0}×Ø", "悬伸 {0}×Ø", "突き出し {0}×Ø", "돌출 {0}×Ø"],
    WStickX => ["Beyond 4×Ø stiffness drops with the cube of the length.", "Oltre 4×Ø la rigidità crolla con il cubo della lunghezza.", "Über 4×Ø sinkt die Steifigkeit mit der dritten Potenz der Länge.", "Au-delà de 4×Ø, la rigidité chute avec le cube de la longueur.", "Más allá de 4×Ø la rigidez cae con el cubo de la longitud.", "超过 4×Ø 后，刚性随长度的三次方下降。", "4×Øを超えると剛性は長さの3乗で低下します。", "4×Ø를 넘으면 강성이 길이의 세제곱으로 떨어집니다."],
    WStickShortT => ["Stickout shorter than the flute length", "Sporgenza minore della lunghezza tagliente", "Auskraglänge kürzer als die Schneidenlänge", "Porte-à-faux plus court que la longueur de coupe", "Voladizo menor que la longitud de corte", "悬伸短于刃长", "突き出しが刃長より短い", "돌출이 날 길이보다 짧음"],
    WStickShortX => ["Check the tool data.", "Controlla i dati dell'utensile.", "Werkzeugdaten prüfen.", "Vérifiez les données de l'outil.", "Revisa los datos de la herramienta.", "请检查刀具数据。", "工具データを確認してください。", "공구 데이터를 확인하세요."],
    WFlutesT => ["{0} flutes slotting aluminium", "{0} taglienti in cava su alluminio", "{0} Schneiden beim Nuten in Aluminium", "{0} dents en rainurage aluminium", "{0} filos ranurando aluminio", "{0} 刃铣铝槽", "{0}枚刃でアルミの溝加工", "{0}날로 알루미늄 슬롯 가공"],
    WFlutesX => ["The gullets clog. Use 2–3 flutes for slotting.", "Le gole si intasano. Per la cava usa 2–3 taglienti.", "Die Spannuten verstopfen. Zum Nuten 2–3 Schneiden verwenden.", "Les goujures se bouchent. Utilisez 2 à 3 dents pour rainurer.", "Las gargantas se obstruyen. Usa 2–3 filos para ranurar.", "容屑槽会堵塞，铣槽请用 2–3 刃。", "溝が詰まります。溝加工には2〜3枚刃を使ってください。", "칩 포켓이 막힙니다. 슬롯 가공은 2–3날을 쓰세요."],
    WDeepSlotT => ["Slot deeper than 1×Ø", "Cava profonda oltre 1×Ø", "Nut tiefer als 1×Ø", "Rainure plus profonde que 1×Ø", "Ranura más profunda que 1×Ø", "槽深超过 1×Ø", "1×Øを超える深溝", "1×Ø보다 깊은 슬롯"],
    WDeepSlotX => ["Chip evacuation is hard. Use air or coolant, or a trochoidal strategy.", "Evacuazione truciolo difficile. Usa aria o refrigerante, oppure una strategia trocoidale.", "Spanabfuhr schwierig. Luft oder Kühlmittel verwenden oder trochoidal fräsen.", "Évacuation des copeaux difficile. Utilisez air ou lubrifiant, ou une stratégie trochoïdale.", "La evacuación de viruta es difícil. Usa aire o refrigerante, o una estrategia trocoidal.", "排屑困难。请使用吹气或冷却液，或改用摆线策略。", "切りくず排出が困難です。エアかクーラント、またはトロコイド加工を使ってください。", "칩 배출이 어렵습니다. 에어나 절삭유, 또는 트로코이달 전략을 쓰세요."],
    WCapT => ["Spindle limits the speed", "Il mandrino limita i giri", "Spindel begrenzt die Drehzahl", "La broche limite la vitesse", "El husillo limita la velocidad", "主轴转速受限", "主軸回転数が上限", "주축 회전수 제한"],
    WCapX => ["Actual Vc {0} m/min ({1}% of recommended). Chip load stays correct; you can raise ae instead of speed.", "Vc reale {0} m/min ({1}% del consigliato). Il truciolo resta corretto; puoi alzare ae invece della velocità.", "Tatsächliche Vc {0} m/min ({1} % der Empfehlung). Die Spandicke bleibt richtig; erhöhe ae statt der Drehzahl.", "Vc réelle {0} m/min ({1} % de la recommandée). Le copeau reste correct ; augmentez ae plutôt que la vitesse.", "Vc real {0} m/min ({1} % de la recomendada). La viruta sigue siendo correcta; puedes subir ae en lugar de la velocidad.", "实际 Vc {0} m/min（推荐值的 {1}%）。切屑厚度仍然正确，可以加大 ae 来代替提速。", "実Vc {0} m/min（推奨の{1}%）。切りくず厚さは適正なので、回転の代わりにaeを増やせます。", "실제 Vc {0} m/min (권장의 {1}%). 칩 두께는 적정하므로 속도 대신 ae를 늘릴 수 있습니다."],
    WMinT => ["Minimum speed too high", "Giri minimi troppo alti", "Mindestdrehzahl zu hoch", "Vitesse minimale trop élevée", "Velocidad mínima demasiado alta", "最低转速过高", "最低回転数が高すぎます", "최저 회전수가 너무 높음"],
    WMinX => ["Vc {0} m/min against {1} recommended: the edge may burn.", "Vc {0} m/min contro {1} consigliati: rischio di bruciare il tagliente.", "Vc {0} m/min statt {1} empfohlen: Die Schneide kann verbrennen.", "Vc {0} m/min contre {1} recommandés : l'arête risque de brûler.", "Vc {0} m/min frente a {1} recomendados: el filo puede quemarse.", "Vc {0} m/min，推荐 {1}：刃口可能烧损。", "Vc {0} m/min（推奨 {1}）：刃先が焼ける恐れがあります。", "Vc {0} m/min (권장 {1}): 날이 탈 수 있습니다."],
}

/// Material display names in every language.
pub fn material_name(lang: Lang, id: &str) -> &'static str {
    let names: [&'static str; LANG_COUNT] = match id {
        "al6061" => [
            "Aluminium 6061 / 6082",
            "Alluminio 6061 / 6082",
            "Aluminium 6061 / 6082",
            "Aluminium 6061 / 6082",
            "Aluminio 6061 / 6082",
            "铝合金 6061 / 6082",
            "アルミ 6061 / 6082",
            "알루미늄 6061 / 6082",
        ],
        "al7075" => [
            "Aluminium 7075",
            "Alluminio 7075 (Ergal)",
            "Aluminium 7075 (Ergal)",
            "Aluminium 7075",
            "Aluminio 7075",
            "铝合金 7075",
            "アルミ 7075（超々ジュラルミン）",
            "알루미늄 7075",
        ],
        "alcast" => [
            "Cast aluminium (high Si)",
            "Alluminio da fusione (alto Si)",
            "Aluminiumguss (hoher Si-Gehalt)",
            "Aluminium de fonderie (Si élevé)",
            "Aluminio de fundición (alto Si)",
            "铸造铝合金（高硅）",
            "アルミ鋳物（高Si）",
            "주조 알루미늄 (고Si)",
        ],
        "brass" => [
            "Brass CW614N",
            "Ottone CW614N",
            "Messing CW614N",
            "Laiton CW614N",
            "Latón CW614N",
            "黄铜 CW614N",
            "黄銅 CW614N",
            "황동 CW614N",
        ],
        "bronze" => [
            "Tin bronze CuSn",
            "Bronzo CuSn",
            "Zinnbronze CuSn",
            "Bronze CuSn",
            "Bronce CuSn",
            "锡青铜 CuSn",
            "青銅 CuSn",
            "청동 CuSn",
        ],
        "copper" => [
            "Copper Cu-ETP",
            "Rame Cu-ETP",
            "Kupfer Cu-ETP",
            "Cuivre Cu-ETP",
            "Cobre Cu-ETP",
            "纯铜 Cu-ETP",
            "銅 Cu-ETP",
            "구리 Cu-ETP",
        ],
        "s235" => [
            "Mild steel S235 / C15",
            "Acciaio dolce S235 / C15",
            "Baustahl S235 / C15",
            "Acier doux S235 / C15",
            "Acero dulce S235 / C15",
            "低碳钢 S235 / C15",
            "軟鋼 S235 / C15",
            "연강 S235 / C15",
        ],
        "c45" => [
            "Carbon steel C45 (1045)",
            "Acciaio C45",
            "Vergütungsstahl C45",
            "Acier C45",
            "Acero C45",
            "中碳钢 C45",
            "炭素鋼 C45（S45C）",
            "탄소강 C45 (S45C)",
        ],
        "42crmo4" => [
            "42CrMo4 (4140) ~30 HRC",
            "42CrMo4 bonificato ~30 HRC",
            "42CrMo4 vergütet ~30 HRC",
            "42CrMo4 traité ~30 HRC",
            "42CrMo4 bonificado ~30 HRC",
            "42CrMo4 调质 ~30 HRC",
            "42CrMo4（SCM440）調質 ~30 HRC",
            "42CrMo4 (SCM440) 조질 ~30 HRC",
        ],
        "p20" => [
            "Mould steel 1.2311 (P20)",
            "Acciaio per stampi 1.2311",
            "Formenstahl 1.2311",
            "Acier pour moules 1.2311",
            "Acero para moldes 1.2311",
            "模具钢 1.2311 (P20)",
            "金型鋼 1.2311（P20）",
            "금형강 1.2311 (P20)",
        ],
        "ss304" => [
            "Stainless AISI 304",
            "Inox AISI 304",
            "Edelstahl 1.4301 (304)",
            "Inox AISI 304",
            "Inoxidable AISI 304",
            "不锈钢 304",
            "ステンレス SUS304",
            "스테인리스 304",
        ],
        "ss316" => [
            "Stainless AISI 316",
            "Inox AISI 316",
            "Edelstahl 1.4401 (316)",
            "Inox AISI 316",
            "Inoxidable AISI 316",
            "不锈钢 316",
            "ステンレス SUS316",
            "스테인리스 316",
        ],
        "ss174" => [
            "Stainless 17-4PH",
            "Inox 17-4PH",
            "Edelstahl 17-4PH (1.4542)",
            "Inox 17-4PH",
            "Inoxidable 17-4PH",
            "不锈钢 17-4PH",
            "ステンレス SUS630（17-4PH）",
            "스테인리스 17-4PH",
        ],
        "gg25" => [
            "Grey cast iron EN-GJL-250",
            "Ghisa grigia EN-GJL-250",
            "Grauguss EN-GJL-250",
            "Fonte grise EN-GJL-250",
            "Fundición gris EN-GJL-250",
            "灰铸铁 HT250",
            "ねずみ鋳鉄 FC250",
            "회주철 GC250",
        ],
        "ggg50" => [
            "Ductile iron EN-GJS-500",
            "Ghisa sferoidale EN-GJS-500",
            "Sphäroguss EN-GJS-500",
            "Fonte GS EN-GJS-500",
            "Fundición nodular EN-GJS-500",
            "球墨铸铁 QT500",
            "球状黒鉛鋳鉄 FCD500",
            "구상흑연주철 GCD500",
        ],
        "ti64" => [
            "Titanium Ti6Al4V (grade 5)",
            "Titanio Ti6Al4V (grado 5)",
            "Titan Ti6Al4V (Grade 5)",
            "Titane Ti6Al4V (grade 5)",
            "Titanio Ti6Al4V (grado 5)",
            "钛合金 TC4 (Ti6Al4V)",
            "チタン Ti6Al4V（64チタン）",
            "티타늄 Ti6Al4V (Grade 5)",
        ],
        "in718" => [
            "Inconel 718",
            "Inconel 718",
            "Inconel 718",
            "Inconel 718",
            "Inconel 718",
            "镍基合金 Inconel 718",
            "インコネル 718",
            "인코넬 718",
        ],
        "h50" => [
            "Hardened steel 48–55 HRC",
            "Acciaio temprato 48–55 HRC",
            "Gehärteter Stahl 48–55 HRC",
            "Acier trempé 48–55 HRC",
            "Acero templado 48–55 HRC",
            "淬硬钢 48–55 HRC",
            "焼入れ鋼 48–55 HRC",
            "경화강 48–55 HRC",
        ],
        "pom" => [
            "POM / Delrin",
            "POM / Delrin",
            "POM",
            "POM / Delrin",
            "POM / Delrin",
            "聚甲醛 POM",
            "POM（ジュラコン）",
            "POM (아세탈)",
        ],
        "pmma" => [
            "Acrylic PMMA",
            "Plexiglass PMMA",
            "Acrylglas PMMA",
            "PMMA (plexiglas)",
            "Metacrilato PMMA",
            "亚克力 PMMA",
            "アクリル PMMA",
            "아크릴 PMMA",
        ],
        "hdpe" => [
            "HDPE / PP",
            "HDPE / PP",
            "HDPE / PP",
            "PEHD / PP",
            "HDPE / PP",
            "HDPE / PP",
            "HDPE / PP",
            "HDPE / PP",
        ],
        "g10" => [
            "G10 / FR4 fibreglass",
            "Vetronite G10 / FR4",
            "GFK G10 / FR4",
            "Verre-époxy G10 / FR4",
            "Fibra de vidrio G10 / FR4",
            "环氧玻纤板 G10 / FR4",
            "ガラエポ G10 / FR4",
            "유리에폭시 G10 / FR4",
        ],
        "hardwood" => [
            "Hardwood (beech, oak)",
            "Legno duro (faggio, rovere)",
            "Hartholz (Buche, Eiche)",
            "Bois dur (hêtre, chêne)",
            "Madera dura (haya, roble)",
            "硬木（山毛榉、橡木）",
            "広葉樹（ブナ、オーク）",
            "활엽수 (너도밤나무, 참나무)",
        ],
        "softwood" => [
            "Softwood (fir, pine)",
            "Legno tenero (abete, pino)",
            "Weichholz (Tanne, Kiefer)",
            "Bois tendre (sapin, pin)",
            "Madera blanda (abeto, pino)",
            "软木（冷杉、松木）",
            "針葉樹（モミ、マツ）",
            "침엽수 (전나무, 소나무)",
        ],
        "mdf" => [
            "MDF / plywood",
            "MDF / multistrato",
            "MDF / Sperrholz",
            "MDF / contreplaqué",
            "MDF / contrachapado",
            "中密度板 / 胶合板",
            "MDF / 合板",
            "MDF / 합판",
        ],
        _ => return "?",
    };
    names[lang.idx()]
}

impl Lang {
    pub fn iso(self, g: Iso) -> &'static str {
        self.t(match g {
            Iso::N => K::IsoN,
            Iso::P => K::IsoP,
            Iso::M => K::IsoM,
            Iso::K => K::IsoK,
            Iso::S => K::IsoS,
            Iso::H => K::IsoH,
            Iso::O => K::IsoO,
            Iso::W => K::IsoW,
        })
    }

    pub fn tool_kind(self, k: ToolKind) -> &'static str {
        self.t(match k {
            ToolKind::Flat => K::KindFlat,
            ToolKind::Ball => K::KindBall,
            ToolKind::Bull => K::KindBull,
            ToolKind::VBit => K::KindVBit,
            ToolKind::Face => K::KindFace,
            ToolKind::Drill => K::KindDrill,
        })
    }

    pub fn tool_material(self, m: ToolMaterial) -> &'static str {
        self.t(match m {
            ToolMaterial::Carbide => K::MatCarbide,
            ToolMaterial::HssCo => K::MatHssCo,
            ToolMaterial::Hss => K::MatHss,
        })
    }

    pub fn tool_material_short(self, m: ToolMaterial) -> &'static str {
        match m {
            ToolMaterial::Carbide => self.t(K::MatCarbideShort),
            ToolMaterial::HssCo => "HSS-Co",
            ToolMaterial::Hss => self.t(K::MatHss),
        }
    }

    pub fn coating(self, c: Coating) -> &'static str {
        match c {
            Coating::None => self.t(K::CoatNone),
            Coating::TiN => "TiN",
            Coating::TiCN => "TiCN",
            Coating::AlTiN => "AlTiN / TiAlN",
            Coating::ZrN => "ZrN / DLC",
            Coating::Diamond => self.t(K::CoatDiamond),
        }
    }

    pub fn rigidity(self, r: Rigidity) -> &'static str {
        self.t(match r {
            Rigidity::Low => K::RigLow,
            Rigidity::Medium => K::RigMedium,
            Rigidity::High => K::RigHigh,
        })
    }

    pub fn preset(self, p: MachinePreset) -> &'static str {
        self.t(match p {
            MachinePreset::Router => K::PresetRouter,
            MachinePreset::Bench => K::PresetBench,
            MachinePreset::Vmc => K::PresetVmc,
            MachinePreset::Custom => K::PresetCustom,
        })
    }

    pub fn strategy(self, s: Strategy) -> &'static str {
        self.t(match s {
            Strategy::Slot => K::StratSlot,
            Strategy::Rough => K::StratRough,
            Strategy::Hsm => K::StratHsm,
            Strategy::Finish => K::StratFinish,
        })
    }

    pub fn limiter(self, l: Limiter) -> &'static str {
        self.t(match l {
            Limiter::RpmMax => K::LimRpmMax,
            Limiter::RpmMin => K::LimRpmMin,
            Limiter::FeedMax => K::LimFeedMax,
            Limiter::Power => K::LimPower,
            Limiter::Deflection => K::LimDeflection,
        })
    }

    /// Label and value text of one breakdown row.
    pub fn row(self, r: &Row) -> (String, String) {
        let n = |v: f64, d: usize| self.num(v, d);
        let label = match r.step {
            Step::VcBase(m) => self.f(K::StepVcBase, &[self.tool_material_short(m)]),
            Step::Drilling => self.t(K::StepDrilling).into(),
            Step::Coating(c) => self.f(K::StepCoating, &[self.coating(c)]),
            Step::FullSlot => self.t(K::StepFullSlot).into(),
            Step::Engagement { pct, cooler } => self.f(
                if cooler { K::StepEngCool } else { K::StepEng },
                &[&n(pct, 0)],
            ),
            Step::LifeTweak => self.t(K::StepLife).into(),
            Step::VcTarget => self.t(K::StepVcTarget).into(),
            Step::EffectiveDiameter => self.t(K::StepEffD).into(),
            Step::FzBase { cf } => self.f(K::StepFzBase, &[&n(cf, 4)]),
            Step::FzBaseInsert => self.t(K::StepFzInsert).into(),
            Step::FzFactors {
                tool,
                rigidity,
                tweak,
                slot,
            } => {
                let mut s = self.f(
                    K::StepFzFactors,
                    &[&n(tool, 2), &n(rigidity, 2), &n(tweak, 2)],
                );
                if slot {
                    s.push_str(&self.f(K::StepSlotSuffix, &[&n(0.85, 2)]));
                }
                s
            }
            Step::ChipTarget => self.t(K::StepChipTarget).into(),
            Step::RadialThinning => self.t(K::StepRadialThin).into(),
            Step::BallThinning => self.t(K::StepBallThin).into(),
            Step::LeadThinning { kappa } => self.f(K::StepLeadThin, &[&n(kappa, 0)]),
            Step::ThinningCapped => self.t(K::StepCapped).into(),
            Step::Reduction(l) => self.f(K::StepReduction, &[self.limiter(l)]),
            Step::FzProgrammed => self.t(K::StepFzProg).into(),
            Step::FnBase { cf } => self.f(K::StepFnBase, &[&n(cf, 4)]),
            Step::FnFactors => self.t(K::StepFnFactors).into(),
            Step::FnTarget => self.t(K::StepFnTarget).into(),
        };
        let value = match (r.step, r.unit) {
            (Step::ThinningCapped, _) => String::new(),
            (_, Unit::MPerMin) => format!("{} m/min", n(r.value, r.decimals)),
            (_, Unit::Mm) => format!("{} mm", n(r.value, r.decimals)),
            (_, Unit::MmPerRev) => format!("{} {}", n(r.value, r.decimals), self.t(K::UnitMmRev)),
            (_, Unit::Factor) => format!("×{}", n(r.value, r.decimals)),
            (_, Unit::None) => n(r.value, r.decimals),
        };
        (label, value)
    }

    /// Title and body text of a warning.
    pub fn warning(self, w: &Warning, material_id: &str) -> (String, String) {
        let n = |v: f64, d: usize| self.num(v, d);
        let tx = |t: K, x: K| (self.t(t).to_string(), self.t(x).to_string());
        match w.msg {
            Msg::HssNotSuitable => (
                self.t(K::WHssT).into(),
                self.f(K::WHssX, &[material_name(self, material_id)]),
            ),
            Msg::AlTiNOnAluminium => tx(K::WAltinT, K::WAltinX),
            Msg::DiamondOnFerrous => tx(K::WDiaT, K::WDiaX),
            Msg::PowerShort {
                need_kw,
                avail_kw,
                rpm,
            } => (
                self.t(K::WPowT).into(),
                self.f(K::WPowX, &[&n(need_kw, 2), &n(avail_kw, 2), &n(rpm, 0)]),
            ),
            Msg::DeepHole { ratio, peck } => (
                self.f(K::WDeepHoleT, &[&n(ratio, 1)]),
                self.f(K::WDeepHoleX, &[&n(peck, 1)]),
            ),
            Msg::Over8D => tx(K::WOver8T, K::WOver8X),
            Msg::VBitTooDeep { ap } => (self.t(K::WVbitT).into(), self.f(K::WVbitX, &[&n(ap, 2)])),
            Msg::ThinChip { chip, metal } => (
                self.f(K::WThinT, &[&n(chip, 4)]),
                self.t(if metal {
                    K::WThinMetalX
                } else {
                    K::WThinPlasticX
                })
                .into(),
            ),
            Msg::DeflectionHalvesFeed => tx(K::WDeflHalfT, K::WDeflHalfX),
            Msg::PowerHalvesFeed => tx(K::WPowHalfT, K::WPowHalfX),
            Msg::Deflection { um, limit_um } => (
                self.f(K::WDeflT, &[&n(um, 0)]),
                self.f(K::WDeflX, &[&n(limit_um, 0)]),
            ),
            Msg::ApOverFlute { flute } => (
                self.t(K::WApFluteT).into(),
                self.f(K::WApFluteX, &[&n(flute, 1)]),
            ),
            Msg::LongStickout { ratio } => (
                self.f(K::WStickT, &[&n(ratio, 1)]),
                self.t(K::WStickX).into(),
            ),
            Msg::StickoutShorterThanFlute => tx(K::WStickShortT, K::WStickShortX),
            Msg::TooManyFlutesInAlSlot { z } => (
                self.f(K::WFlutesT, &[&z.to_string()]),
                self.t(K::WFlutesX).into(),
            ),
            Msg::DeepSlot => tx(K::WDeepSlotT, K::WDeepSlotX),
            Msg::RpmCapped { vc, pct } => (
                self.t(K::WCapT).into(),
                self.f(K::WCapX, &[&n(vc, 0), &n(pct, 0)]),
            ),
            Msg::RpmMinTooHigh { vc, target } => (
                self.t(K::WMinT).into(),
                self.f(K::WMinX, &[&n(vc, 0), &n(target, 0)]),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calc::{Setup, compute};

    #[test]
    fn number_formats_per_language() {
        assert_eq!(Lang::En.num(24000.0, 0), "24,000");
        assert_eq!(Lang::It.num(1234567.891, 2), "1.234.567,89");
        assert_eq!(Lang::De.num(0.0456, 4), "0,0456");
        assert_eq!(Lang::Fr.num(12345.5, 1), "12 345,5");
        assert_eq!(Lang::Ja.num(-3.5, 1), "-3.5");
    }

    #[test]
    fn locale_detection() {
        assert_eq!(Lang::from_locale("it-IT"), Lang::It);
        assert_eq!(Lang::from_locale("zh-Hans-CN"), Lang::Zh);
        assert_eq!(Lang::from_locale("pt-BR"), Lang::En);
        assert_eq!(Lang::from_locale("ja_JP.UTF-8"), Lang::Ja);
    }

    #[test]
    fn placeholders_are_filled_everywhere() {
        let mut s = Setup::default();
        s.machine.feed_max = 500.0;
        s.tool.stickout = 60.0;
        let o = compute(&s);
        for lang in Lang::ALL {
            for r in &o.why {
                let (l, v) = lang.row(r);
                assert!(!l.contains('{') && !v.contains('{'), "{lang:?}: {l} / {v}");
            }
            for w in &o.warnings {
                let (t, x) = lang.warning(w, &s.material_id);
                assert!(!t.contains('{') && !x.contains('{'), "{lang:?}: {t} / {x}");
            }
            for m in MATERIALS {
                assert_ne!(material_name(lang, m.id), "?");
            }
        }
    }
}
