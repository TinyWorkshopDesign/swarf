# Roadmap

## Next update: align with a well-known machinists' calculator

_Started 15 Oct 2026, comparison done the same day_

Reference: a well-known commercial speeds and feeds calculator for machinists (its free web version), used as a benchmark. Below it is called *the reference*.

Goal: bring Swarf closer to the reference's cutting model and user experience, keeping the engine in `src/calc.rs` pure and tested.

> **Licence.** The reference's licence forbids reverse engineering, derivative works and reusing its data tables, including in free products. It is used **only as a black-box benchmark**: we compare results and take ideas from its features, but we do not copy its tables, coefficients or formulas into `src/data.rs` or `src/calc.rs`. Every correction to Swarf must come from public sources (handbooks, tool makers' catalogues, Sandvik/Kennametal guides) or from real cuts. Its raw numbers stay in a local folder that is not published; this file contains only ratios and conclusions.

### What the reference has that Swarf lacks

**Inputs and interface**
- [ ] Editable **material hardness** (HB), shown per material and used in the calculation.
- [ ] **mm / inch** units (Swarf is metric only).
- [ ] **Two-way editing**: you can type Vc, fz, RPM or feed and the rest is recomputed.
- [ ] Separate **HSM** toggle that raises Vc at low engagement (in the tests: +67 %). Swarf applies a boost automatically (up to +40 %) with no way to switch it off.
- [ ] **Slotting** toggle, and WOC entered in mm or in % of Ø.
- [ ] **Helix angle** of the tool (the reference also reports an "axial thinning").
- [ ] Automatic suggestion of DOC/WOC, stickout and flute length when you change material or Ø (Swarf has strategy buttons, which is similar).
- [ ] **Safe torque** output (torque the shank can take) next to required torque.
- [ ] Print report and share link (Swarf has saved setups, which is similar).

**Catalogue**
- [ ] Tools: roughing (HP) end mill, high-feed mill, chamfer mill, T-slot, corner rounding, lens end mill, indexed (end mill, ball, helical); spot drill, parabolic drill, spade drill, indexed drill, reamer, counterbore, countersink, boring head; thread mill, cutting tap, forming tap; turning (profiling, facing, grooving, boring).
- [ ] Tool materials: **PCD** and **ceramic** (Swarf: carbide, HSS-Co, HSS).
- [ ] Coatings: **TiAlN**, **AlCrN**, nACRo, nACo (Swarf: TiN, TiCN, AlTiN, ZrN, diamond).
- [ ] Materials: about 40 groups in the free version (over 200 in the paid one), split by hardness (e.g. 7 stainless families, 3 titanium families). Swarf has 25.

**Extra calculators** (low priority, outside the core): tap and drill tables, threads, ISO fits, bolt-hole circles, trigonometry, G/M-code reference.

**Where Swarf is already ahead** (to keep): machine with power curve and limits on power, feed and deflection; tool deflection; "where the numbers come from" breakdown; MRR/power charts and strategy comparison; ready G-code; 8 languages; offline desktop app, open source.

### Comparison of results

**Method.** 20 milling cases and 3 drilling cases, computed in both apps with the same inputs: carbide uncoated (unless noted), flute length 3·Ø, stickout 4·Ø, shank = Ø, chip thinning on, the reference HSM off. For Swarf: a machine with **no limits** (rpm, power and feed) and rigidity "High", so we compare the cutting model, not the machine. Matching materials as close as possible (the reference has material groups, not alloys). The Swarf cases are computed by a small local runner on `swarf::calc` (not published).

| Case | Material (Swarf ↔ reference) | Tool | ap × ae (mm) |
|---|---|---|---|
| C1 | al6061 ↔ 6061-T6 | Ø10 z3 | 10 × 5 (50 %) |
| C2 | al6061 | Ø10 z3 | 5 × 10 (slot) |
| C3 | al6061 | Ø10 z3 | 20 × 1 (10 %, HSM) |
| C4 | al6061 | Ø6 z2 | 6 × 1.2 |
| C5 | al6061 | Ø3 z2 | 1.5 × 3 (slot) |
| C6 | s235 ↔ A36 | Ø10 z4 | 10 × 3 |
| C7 | c45 ↔ structural steels <0.5 %C | Ø10 z4 | 5 × 10 (slot) |
| C8 | 42crmo4 ↔ 4140 300 HB | Ø12 z4 | 12 × 1.2 |
| C9 | ss304 ↔ austenitic stainless | Ø10 z4 | 10 × 2 |
| C10 | gg25 ↔ cast iron | Ø10 z4 | 10 × 3 |
| C11 | ti64 ↔ titanium α/α-β | Ø10 z4 | 10 × 1 |
| C12 | in718 ↔ Ni/Co superalloys | Ø10 z4 | 10 × 1 |
| C13 | brass ↔ copper alloys | Ø6 z2 | 6 × 2 |
| C14 | pom ↔ plastics | Ø6 z2 | 6 × 6 (slot) |
| C15 | hardwood ↔ wood | Ø6 z2 | 6 × 6 (slot) |
| C16 | al6061 | ball nose Ø6 z2 | 0.5 × 0.5 |
| C17 | s235 | ball nose Ø10 z2 | 0.3 × 0.4 |
| C18 | al6061 | Ø20 z3 | 20 × 8 |
| C19 | s235, **HSS** | Ø10 z4 | 10 × 3 |
| C20 | al6061, **ZrN** | Ø10 z3 | 10 × 5 |

**Swarf / reference ratios** (1.00 = same; > 1 = Swarf higher)

| Case | Vc | fz programmed | Feed | Spindle power |
|---|---|---|---|---|
| C1 | 1.15 | 1.85 | 2.12 | 5.1 |
| C2 | 1.22 | 1.91 | 2.34 | 5.8 |
| C3 | 1.52 | 0.88 | 1.34 | 3.4 |
| C4 | 1.41 | 0.96 | 1.35 | 3.9 |
| C5 | 1.22 | 1.90 | 2.32 | 7.6 |
| C6 | 1.52 | 1.11 | 1.69 | 4.5 |
| C7 | 1.24 | 2.59 | 3.21 | 9.6 |
| C8 | 2.41 | 0.73 | 1.77 | 4.2 |
| C9 | 2.31 | 1.10 | 2.55 | 3.5 |
| C10 | 1.61 | 1.59 | 2.56 | 4.9 |
| C11 | 2.17 | 0.88 | 1.90 | 1.7 |
| C12 | 1.95 | 1.05 | 2.05 | 2.9 |
| C13 | 1.23 | 2.50 | 3.07 | 2.5 |
| C14 | 1.61 | 1.17 | 1.89 | 12.8 |
| C15 | 0.91 | 1.30 | 1.19 | 9.6 |
| C16 | 1.47 | 0.96 | 1.40 | 3.5 |
| C17 | 1.75 | 0.73 | 1.28 | 3.0 |
| C18 | 1.23 | 1.11 | 1.37 | 3.0 |
| C19 | 1.07 | 1.10 | 1.18 | 3.2 |
| C20 | 1.35 | 1.85 | 2.48 | 6.0 |
| **Geometric mean** | **1.47** | **1.27** | **1.86** | **4.4** |
| Range | 0.91–2.41 | 0.73–2.59 | 1.18–3.21 | 1.7–12.8 |

| Drill | Material | Vc | fn | Feed | Power |
|---|---|---|---|---|---|
| D1 carbide Ø8 × 24 | c45 | 1.28 | 0.94 | 1.20 | 3.6 |
| D2 HSS Ø6.8 × 20 | al6061 | 0.77 | 0.72 | 0.55 | 1.5 |
| D3 carbide Ø10 × 30 | ss304 | 2.08 | 0.68 | 1.41 | 1.9 |

### What the comparison shows

1. **Geometry matches.** Radial chip thinning gives the same factors (1.0206, 1.0607, 1.0911, 1.25, 1.6667), and so do the ball nose's effective diameter (3.32 / 3.41 mm) and its thinning (2.528 / 4.555). This part of Swarf is consistent.
2. **Swarf is more aggressive on feed: on average 1.9×, up to 3.2×.** It is the sum of the two points below.
3. **Vc is higher in Swarf (on average 1.5×), much higher on difficult materials:** stainless 2.3×, titanium 2.2×, Inconel 1.95×, 4140 2.4× (but the reference uses 4140 at 300 HB, so the match is not fair). Part of the difference is Swarf's automatic boost at low engagement (up to +40 %). The reference does it only when you tick HSM (+67 %; with HSM on, C3 and C11 come to 0.91× and 1.30×). In the reference, Vc also falls with engagement: 100 % at ≤ 20 % ae, about 94 % at 50 %, 80 % in a slot. Swarf uses 100 % at 50 % and 90 % in a slot.
4. **The target chip in the reference depends on engagement; in Swarf it is almost constant.** With the same Ø10 in aluminium, the reference goes from 0.114 mm (ae 10 %) to 0.054 mm (ae 50 %) to 0.044 mm (slot), roughly halving the chip as the cut gets heavier. Swarf stays at 0.10 → 0.10 → 0.085. This is why Swarf is about 2× higher in C1, C2, C7, C13 and C20 (heavy cuts) and about equal or lower in C3, C4, C8, C11 (light cuts).
5. **Small tools:** at Ø3 in a slot the reference gives fz 0.013 against Swarf's 0.026. The reference's chip shrinks faster with diameter.
6. **Power: Swarf is on average 4.4× higher.** Checking with handbook unit power: in aluminium (≈ 0.7–1.0 J/mm³) the reference gives 0.68 J/mm³ and Swarf 1.39 J/mm³ at the tool. In mild steel (≈ 2.7–3.8 J/mm³) Swarf gives 3.5 and the reference 1.5. So Swarf looks high on aluminium and plastics/wood (kc1.1 or mc too high for soft materials), while the reference looks low on steel. **To verify with published examples (Sandvik, Kennametal) before changing anything.**
7. **Coatings:** ZrN on aluminium raises Vc by 20 % in Swarf, only 3 % in the reference.
8. **Drills:** the feed per rev is close in steel (0.94×) but Swarf is lower in aluminium HSS (0.72×) and in stainless (0.68×). Vc in stainless with carbide is 2× in Swarf.
9. **HSS** (C19): the closest match of all (Vc 1.07×, feed 1.18×).

### Alignment done (15 Oct 2026)

**Rule followed.** Where the reference agrees with the tool makers we align with it; where it contradicts them we follow **Kennametal and Sandvik Coromant**, whose data are public. Every value comes from these sources, none is copied from the reference.

**Sources checked**
- Kennametal, GOmill™ PRO solid carbide end mills, *Application Data* tab: speed (Kv) and feed (KFz) factors versus ae/D1, and starting Vc and fz per material group and diameter.
- Sandvik Coromant, *Metalcutting Technical Guide – Milling* (pp. D22–D26): formulas for hm, kc and Pc with efficiency, rake angle correction (1 % per degree), table of constant K for power per CMC, Vc factors versus hardness.
- Sandvik Coromant, *Formulas and definitions for milling – metric*.

**What changed in Swarf**
- [x] **HSM option** (checkbox in the Cut section, turned on by the "Trochoidal / adaptive" strategy). Without HSM, Vc no longer rises at low engagement (before it did automatically, up to +40 %). With HSM it follows Kennametal's Kv curve: ×1.1 at 40 %, ×1.2 at 30 %, ×1.3 at 20 %, ×1.4 at 10 %, ×1.6 at 4–8 %, ×2.1 at 2 % (lower end of the ranges).
- [x] **Slot**: fz −20 % (Kennametal), it was −15 %. Vc in a slot stays ×0.9.
- [x] **Power**: Sandvik rake angle correction `kc·(1 − γ/100)` with a typical end mill γ per group (N, O, W 15°; P, M, S 8°; K 5°; H 0°). kc1.1 recalibrated on Sandvik's K table: 6061 650, 7075 700, brass 700, bronze and copper 1500, C45 1600, 17-4PH 2600, Ti6Al4V 1400, hardened steel 4000.
- [x] **Material hardness**: new input (HB, or HRC for hardened steel) with Sandvik's Vc factors (e.g. low-alloy steel +40 HB → ×0.91). Available where Sandvik gives a table: steels, stainless, cast irons, superalloys, hardened steel. Not for aluminium, copper, titanium, plastics, wood.
- [x] 5 new tests (HSM, HSM strategy, slot, hardness, power against the Sandvik example).

**Check against Kennametal** (Swarf with AlTiN, rigid machine, Ø10, side milling at 0.4·D, no HSM; Kennametal GOmill PRO TiN/TiAlN, "start" value)

| Material | Vc Swarf | Vc Kennametal (range) | fz Swarf | fz Kennametal |
|---|---|---|---|---|
| S235 ↔ P1 | 203 | 175 (150–200) | 0.060 | 0.072 |
| C45 ↔ P2 | 176 | 165 (140–190) | 0.055 | 0.072 |
| 42CrMo4 ↔ P3 | 135 | 140 (120–160) | 0.050 | 0.061 |
| 304 ↔ M1 | 117 | 103 (90–115) | 0.045 | 0.061 |
| Grey cast iron ↔ K1 | 169 | 135 (120–150) | 0.060 | 0.072 |
| Ti6Al4V ↔ S4 | 63 | 55 (50–60) | 0.040 | 0.045 |
| Inconel 718 ↔ S3 | 38 | 33 (25–40) | 0.030 | 0.032 |
| Hardened 48–55 HRC ↔ H2 | 91 | 95 (70–120) | 0.025 | 0.040 |

Vc is in line (grey cast iron, stainless and S235 slightly above the top of the range). fz is 10–25 % below Kennametal (conservative), 37 % below on hardened steel. The scaling with diameter matches (Kennametal Ø20/Ø10 = 1.58, Swarf 1.55; Ø3/Ø10 = 0.29, Swarf 0.30). Kennametal's KFz factor versus engagement is exactly the radial thinning formula already in Swarf.

**Check against Sandvik (power).** With the rake angle correction, Swarf matches power within ±5 % on the cases checked: C1 aluminium 4.63 kW against 4.47 from Sandvik's K table; C6 S235 2.37 kW against 2.48. A test in `calc.rs` compares kc with Sandvik's value for CMC 01.1.

**New Swarf / reference ratios** (same cases as above)

| Case | Vc | fz | Feed | Power |
|---|---|---|---|---|
| C1 | 1.15 | 1.85 | 2.12 | 4.0 |
| C2 | 1.22 | 1.80 | 2.20 | 4.4 |
| C3 | 1.09 | 0.88 | 0.96 | 1.9 |
| C4 | 1.09 | 0.96 | 1.04 | 2.4 |
| C5 | 1.22 | 1.79 | 2.18 | 5.8 |
| C6 | 1.27 | 1.11 | 1.41 | 3.5 |
| C7 | 1.24 | 2.44 | 3.02 | 8.0 |
| C8 | 1.72 | 0.73 | 1.26 | 2.7 |
| C9 | 1.78 | 1.10 | 1.96 | 2.5 |
| C10 | 1.34 | 1.59 | 2.13 | 3.9 |
| C11 | 1.55 | 0.88 | 1.36 | 1.1 |
| C12 | 1.40 | 1.05 | 1.47 | 1.9 |
| C13 | 1.06 | 2.50 | 2.63 | 1.6 |
| C14 | 1.61 | 1.10 | 1.78 | 10.3 |
| C15 | 0.91 | 1.23 | 1.12 | 7.8 |
| C16 | 1.09 | 0.96 | 1.04 | 2.0 |
| C17 | 1.27 | 0.73 | 0.93 | 2.0 |
| C18 | 1.12 | 1.11 | 1.24 | 2.1 |
| C19 | 0.90 | 1.10 | 0.98 | 2.5 |
| C20 | 1.35 | 1.85 | 2.48 | 4.7 |
| C3 + HSM | 0.91 | 0.88 | 0.80 | 1.6 |
| C11 + HSM | 1.30 | 0.88 | 1.14 | 0.9 |
| **Geometric mean** | **1.23** (was 1.47) | **1.21** (was 1.27) | **1.49** (was 1.86) | **2.9** (was 4.4) |

**Remaining differences, and why we don't follow the reference there**
- *Chip that shrinks with engagement* (C1, C2, C5, C7, C13, C20): the reference halves the chip going from 10 % to 50 % of ae. Kennametal does the opposite: its base fz is for side milling at 0.4·D and full flute length, and only the thinning compensation changes with ae. We stay with Kennametal.
- *Vc on difficult materials* (C8, C9, C11, C12): the reference is 30–45 % below Kennametal's ranges; Swarf is inside them.
- *Power*: The reference is about half of what Sandvik's K table gives; Swarf is within ±5 %.

### Second calibration (15 Oct 2026)

**Additional sources**
- Kennametal GOdrill™ 3xD (solid carbide drill, coated): Vc (min/start/max) and fn ranges per group and diameter 1–20 mm.
- Kennametal KOR5™ DA (aluminium end mill): fz per diameter at ae 0.5·D and Vc 200–2000 m/min.

**What changed**
- [x] **Drilling Vc** per material group instead of a fixed ×0.7 on the end mill speed: P 0.5, M 0.35, K 0.9, N 0.55, S 0.55, H 0.2 (O, W stay at 0.7). Before, Swarf drilled stainless at 82 m/min with AlTiN, against Kennametal's 20–40; hardened steel at 64 against 10–30.
- [x] **Drilling fn**: factor ×1.5 for cast iron and ×0.8 for stainless on top of 2.5·cf·Ø.
- [x] **fz of the end mills** brought to Kennametal's start values: S235 and C45 0.0070·Ø, 42CrMo4/P20 0.0060, 304 0.0060, 316 0.0055, 17-4PH 0.0048, grey cast iron 0.0070, ductile iron 0.0060, Ti6Al4V 0.0045, Inconel 0.0032, hardened steel 0.0035 (Kennametal 0.0040: kept a little under).
- [x] **Vc** inside Kennametal's ranges: S235 130, C45 125, 304 80, 316 75, 17-4PH 60, grey cast iron 105, ductile iron 95, Ti6Al4V 45 m/min (uncoated carbide base).
- [x] New test `matches_kennametal_tables`: 9 end-mill materials and 10 drill materials must stay inside Kennametal's ranges.

**Aluminium (KOR5).** Kennametal gives fz 0.080 at Ø10, 0.120 at Ø12, 0.160 at Ø16, 0.200 at Ø20, 0.225 at Ø25 (5 flutes, ae 0.5·D). Swarf gives 0.100 / 0.120 / 0.139 / 0.155 / 0.173: +25 % at Ø10, −22 % at Ø20. The average is in line, but the curve against diameter is different (Kennametal grows faster above 12 mm). Left as it is for now; see priorities.

**Swarf / reference ratios after the second calibration**

| | Vc | fz | Feed | Power |
|---|---|---|---|---|
| Geometric mean, milling | **1.18** (1.23) | 1.30 (1.21) | 1.54 (1.49) | 2.9 (2.9) |
| D1 carbide Ø8 C45 | 0.88 (1.28) | 1.20 (0.94) | 1.05 (1.20) | 2.8 |
| D2 HSS Ø6.8 aluminium | 0.60 (0.77) | 0.71 (0.72) | 0.43 (0.55) | 1.1 |
| D3 carbide Ø10 304 | 0.92 (2.08) | 0.73 (0.68) | 0.67 (1.41) | 0.9 |

(in brackets the value before). fz moves away from the reference because Swarf now follows Kennametal, which is above the reference for steels, stainless and cast iron. On drills, Vc for steel and stainless is now close to the reference and inside Kennametal's ranges.

### Priorities

1. [x] ~~Target chip that depends on engagement~~: checked against Kennametal, not needed.
2. [x] HSM option and Vc recalibration with engagement (Kennametal).
3. [x] Power model checked against Sandvik, rake correction and kc1.1.
4. [x] Hardness input (Sandvik factors).
5. [x] fz of P, M, K, S, H brought to Kennametal.
6. [x] Vc of grey cast iron (and S235, stainless, titanium) inside Kennametal's ranges.
7. [ ] **Aluminium**: fz curve against diameter (Kennametal KOR5 grows faster above 12 mm). Find a second source for 2–3 flute aluminium end mills before changing it.
8. [x] Drills: Vc and fn per group from Kennametal GOdrill.
9. [ ] **HSS drills in aluminium**: Swarf 66 m/min, the reference 110. Kennametal's data are for carbide only; find an HSS source.
10. [ ] Rake correction for drilling too (Sandvik drilling formula).
11. [x] **Inch** units and **two-way editing** of Vc / fz / RPM / feed (see below).
12. [ ] New coatings (TiAlN, AlCrN) and tool materials (PCD, ceramic); helix angle.
13. [ ] New tools, in order of usefulness: spot drill, chamfer mill, roughing end mill, high-feed mill, reamer, tap and thread mill.
14. [ ] "Safe torque" output for the shank.

### Inch units and two-way editing (15 Oct 2026)

- [x] **mm / in** toggle next to the language picker, default from the system's region (US, Liberia, Myanmar → inch). The engine stays metric; `src/units.rs` converts every field and result: lengths, Vc in SFM, feed in in/min, fz and fn in in, power in hp, torque in lbf·ft, force in lbf, MRR in in³/min, deflection and scallop in in, Ra in µin. kc1.1 stays in N/mm². G-code in inches starts with G20.
- [x] **Two-way editing** in the display: Vc, fz (f for drills), S and F are editable. Speed (Vc or S) and feed (fz or F) are independent pairs: editing one value fixes it and the other of the pair follows; the last one edited wins. Values set by hand:
  - skip the automatic reductions for power and deflection, but the warnings stay (e.g. "Not enough power");
  - still respect the machine limits (max/min rpm, max feed);
  - show a "set by hand" chip, the recommended S and F, and ↺ to go back;
  - are cleared when material, tool type, tool material or diameter change; comparison charts and strategies always use the recommended values.
- [x] **Screenshot mode** `SWARF_SCREENSHOT=<folder>` to regenerate the README images and to check the layout.
- [x] 7 new tests (conversions, regional default, linked speed/feed, reductions skipped, drills, G-code in inches, saved setups backward compatible).

After each change to the model: run the local comparison again, `cargo test` (the Kennametal and Sandvik tests must stay green) and update the tables.
