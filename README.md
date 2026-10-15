# Swarf

Speeds and feeds calculator for CNC milling machines, for macOS, Linux and Windows.
Written in Rust with [egui](https://github.com/emilk/egui): one executable of about 6 MB, no runtime to install.

## Features

- Spindle speed **S** and feed **F** for flat, ball, bull-nose and V-bit end mills, indexable face mills and twist drills.
- Chip thinning compensation (narrow stepovers, shallow ball-nose cuts, lead angle).
- Power, torque and cutting force (Kienzle model) and tool deflection (two-segment cantilever: shank + flutes).
- Feed is reduced automatically when power or deflection would exceed their limits, or when the machine's max feed is reached; the app shows which limit applied.
- **Comparison charts**:
  - MRR and required/available spindle power while radial engagement `ae` or axial depth `ap` varies;
  - MRR and power bars for the four strategies (full slot, roughing, trochoidal, finishing) and for saved setups.
- Saved setups library, stored automatically on exit.
- Interface in English, Italian, German, French, Spanish, Simplified Chinese, Japanese and Korean. The language follows the system on first launch and can be changed from the menu at the top left.

Material data are typical handbook values (`src/data.rs`). They are a starting point: always check against the tool maker's catalogue.

## Code layout

| File | Contents |
|---|---|
| `src/data.rs` | Materials (ISO groups), coatings, machine presets, strategies |
| `src/calc.rs` | Calculation engine: pure, no UI and no text, with tests |
| `src/i18n.rs` | Translations, number formatting per language |
| `src/fonts.rs` | Loads CJK fonts from the system only when needed |
| `src/app.rs` | egui interface, DRO display, charts |
| `src/main.rs` | Window start-up |
| `assets/` | Icon in every format (SVG source, PNG sizes, `.icns`, `.ico`) and the Linux `.desktop` file |
| `scripts/make_icon.py` | Regenerates every icon file from code (needs `rsvg-convert` and Pillow) |
| `scripts/bundle_macos.sh` | Builds `Swarf.app` with its icon |
| `build.rs` | Embeds the icon in the Windows executable |
| `prototipo/truciolo.html` | First web prototype, kept for reference |

### Adding a language

1. Add a variant to `Lang` in `src/i18n.rs` and to `Lang::ALL`, and raise `LANG_COUNT`.
2. Add one column to every entry of the `strings!` table and to `material_name`. The compiler reports every entry still missing a translation.
3. Set its separators in `Lang::separators` and its locale code in `Lang::from_locale`.

Only left-to-right scripts are supported; egui does not lay out right-to-left text.

## Build and run

Rust is required (https://rustup.rs).

```bash
cargo run --release
```

```bash
cargo test
```

The executable is written to `target/release/swarf` (`swarf.exe` on Windows).

### macOS app bundle

```bash
./scripts/bundle_macos.sh
```

This writes `target/release/Swarf.app`, which can be dragged into Applications. Set `BUNDLE_ID` to your own reverse-domain identifier before distributing it.

### Windows from a Mac

With MinGW installed (`brew install mingw-w64`):

```bash
cargo build --release --target x86_64-pc-windows-gnu
```

### Linux

Linux needs the windowing and OpenGL libraries (Debian/Ubuntu):

```bash
sudo apt install libxkbcommon-dev libgl1-mesa-dev libwayland-dev libx11-dev libxcursor-dev libxrandr-dev libxi-dev
```

For Chinese, Japanese and Korean, install a CJK font such as `fonts-noto-cjk`.

To show the app in the desktop menu, copy `assets/swarf.desktop` to `~/.local/share/applications/` and `assets/icon-256.png` to `~/.local/share/icons/hicolor/256x256/apps/swarf.png`.

### All platforms automatically

`.github/workflows/build.yml` builds and tests on macOS (Apple Silicon and Intel), Linux and Windows on every push to GitHub and attaches the executables to the run.

## Where settings are stored

In the user data folder managed by eframe:

- macOS: `~/Library/Application Support/swarf`
- Linux: `~/.local/share/swarf`
- Windows: `%APPDATA%\swarf\data`
