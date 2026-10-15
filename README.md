<p align="center">
  <img src="assets/icon-256.png" width="128" alt="Swarf icon: an end mill throwing a curled chip">
</p>

<h1 align="center">Swarf</h1>

<p align="center">
  Speeds and feeds calculator for CNC milling and drilling.<br>
  Free, open source, runs on macOS, Linux and Windows.
</p>

<p align="center">
  <a href="https://github.com/TinyWorkshopDesign/swarf/actions/workflows/build.yml"><img src="https://github.com/TinyWorkshopDesign/swarf/actions/workflows/build.yml/badge.svg" alt="Build status"></a>
  <a href="https://github.com/TinyWorkshopDesign/swarf/releases/latest"><img src="https://img.shields.io/github/v/release/TinyWorkshopDesign/swarf" alt="Latest release"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-GPL--3.0-blue" alt="License: GPL-3.0"></a>
</p>

![Results tab: spindle speed and feed on a DRO-style display, with chip thickness, power, torque and deflection](docs/screenshots/results.png)

Pick your machine, material, tool and cut. Swarf gives you spindle speed and feed, explains every factor behind them, and warns you before the tool deflects, the spindle runs out of power or the chip gets too thin to cut.

## Features

- **Speeds and feeds** for flat, ball, bull-nose and V-bit end mills, indexable face mills and twist drills, in 25 materials across the ISO groups (aluminium, steels, stainless, cast iron, titanium, superalloys, hardened steel, plastics, wood).
- **Chip thinning** compensation for narrow stepovers, shallow ball-nose cuts and lead angles.
- **Cutting power, torque and force** from the Kienzle model, with the spindle's real power curve (power drops below base speed).
- **Tool deflection** of shank and flutes, with a limit you set.
- **Automatic limits**: feed is lowered when power, deflection or the machine's max feed would be exceeded, and the display shows which one applied.
- **Where the numbers come from**: every multiplier from table value to programmed feed, step by step.
- **Comparison charts**: material removal rate and spindle power while radial width `ae` or depth `ap` changes, and side-by-side bars for full slot, roughing, trochoidal and finishing strategies plus your saved setups.
- **Ready G-code** (`S… M3`, `G1 F…`, `G81`/`G83` for drilling) with a copy button.
- **Saved setups** library, kept between sessions.
- **8 languages**: English, Italiano, Deutsch, Français, Español, 简体中文, 日本語, 한국어. The language follows your system and can be changed from the menu.

![Comparison charts: MRR and spindle power versus radial engagement, and strategy comparison bars](docs/screenshots/charts.png)

## Download

Get the latest build for your system from the [Releases page](https://github.com/TinyWorkshopDesign/swarf/releases/latest):

| System | File | How to run |
|---|---|---|
| macOS (Apple Silicon and Intel) | `Swarf-…-macos.zip` | Unzip and drag **Swarf** into Applications. The app is not signed: the first time, right-click it and choose **Open**. |
| Windows 10/11 (64-bit) | `Swarf-…-windows-x64.zip` | Unzip and run `swarf.exe`. If SmartScreen warns about an unknown publisher, choose **More info → Run anyway**. |
| Linux (x86-64) | `Swarf-…-linux-x64.tar.gz` | Extract and run `./swarf`. Chinese, Japanese and Korean need a CJK font such as `fonts-noto-cjk`. |

## Build from source

Install Rust 1.95 or newer from [rustup.rs](https://rustup.rs), then:

```bash
git clone https://github.com/TinyWorkshopDesign/swarf.git
```

```bash
cd swarf
```

```bash
cargo run --release
```

On Debian or Ubuntu, install the windowing and OpenGL libraries first:

```bash
sudo apt install libxkbcommon-dev libgl1-mesa-dev libwayland-dev libx11-dev libxcursor-dev libxrandr-dev libxi-dev
```

Run the tests:

```bash
cargo test
```

### Packaging

- **macOS app bundle**: `./scripts/bundle_macos.sh` writes `target/release/Swarf.app`. Set `BUNDLE_ID` to your own reverse-domain identifier when distributing.
- **Windows from a Mac**: with MinGW (`brew install mingw-w64`), `cargo build --release --target x86_64-pc-windows-gnu`. The icon is embedded by `build.rs`.
- **Linux desktop entry**: copy `assets/swarf.desktop` to `~/.local/share/applications/` and `assets/icon-256.png` to `~/.local/share/icons/hicolor/256x256/apps/swarf.png`.
- **Releases**: pushing a tag such as `v0.1.0` runs `.github/workflows/release.yml`, which builds all three platforms and publishes a GitHub release.

## How the numbers are computed

| Step | Model |
|---|---|
| Cutting speed | Handbook Vc per material and tool material, scaled by coating and by radial engagement (narrow cuts run cooler and faster) |
| Feed per tooth | Material factor × diameter, scaled by tool material and machine rigidity |
| Chip thinning | Radial: `1 / (2·√(ae/D − (ae/D)²))` when ae < D/2; lead angle and ball nose: `1 / sin κ` |
| Specific cutting force | Kienzle: `kc = kc1.1 · hm^−mc`, with mean chip thickness `hm` from the engagement angle |
| Power | Milling `Pc = ap·ae·vf·kc / 6·10⁷` kW; drilling `Pc = f·vc·D·kc / 240 000` kW; spindle efficiency 0.85 |
| Deflection | Cantilever of shank (Ø shank) and flutes (0.8·Ø) under the tangential cutting force |

Material data are typical values from machining handbooks, in `src/data.rs`. **They are a starting point.** Check them against your tool maker's catalogue, start conservatively and listen to the cut.

## Code layout

| File | Contents |
|---|---|
| `src/calc.rs` | Calculation engine: pure, no UI and no text, with tests |
| `src/data.rs` | Materials, coatings, machine presets, strategies |
| `src/i18n.rs` | Translations and number formatting per language |
| `src/fonts.rs` | Loads CJK fonts from the system only when needed |
| `src/app.rs` | egui interface, DRO display, charts |
| `src/main.rs` | Window start-up and icon |
| `assets/` | Icon in every format and the Linux `.desktop` file |
| `scripts/make_icon.py` | Regenerates every icon file (needs `rsvg-convert` and Pillow) |
| `scripts/bundle_macos.sh` | Builds `Swarf.app` |
| `prototipo/truciolo.html` | The first web prototype, kept for reference |

## Contributing

Issues and pull requests are welcome, especially:

- cutting data you have verified on real machines, with material, tool and result;
- new materials, coatings or tool types;
- translations and corrections by native speakers.

To add a language: add a variant to `Lang` in `src/i18n.rs` and to `Lang::ALL`, raise `LANG_COUNT`, add one column to every entry of the `strings!` table and to `material_name` (the compiler lists every missing entry), then set its separators and locale code. Only left-to-right scripts are supported.

Please run `cargo fmt` and `cargo test` before opening a pull request.

## Settings location

Settings and saved setups are stored in:

- macOS: `~/Library/Application Support/swarf`
- Linux: `~/.local/share/swarf`
- Windows: `%APPDATA%\swarf\data`

## License

Swarf is free software: you can redistribute it and/or modify it under the terms of the [GNU General Public License](LICENSE) as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Swarf is distributed in the hope that it will be useful, but **without any warranty**; without even the implied warranty of merchantability or fitness for a particular purpose. Cutting parameters are estimates: you are responsible for the safe operation of your machine.
