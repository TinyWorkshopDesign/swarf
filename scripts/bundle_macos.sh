#!/bin/sh
# Builds target/release/Swarf.app with the icon, ready to drag into /Applications.
# BIN=path/to/swarf uses an already built binary (CI passes a universal one).
set -e
cd "$(dirname "$0")/.."
BUNDLE_ID="${BUNDLE_ID:-io.github.swarf}"
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)

if [ -z "$BIN" ]; then
  cargo build --release
  BIN=target/release/swarf
fi
APP=target/release/Swarf.app
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BIN" "$APP/Contents/MacOS/swarf"
cp assets/icon.icns "$APP/Contents/Resources/Swarf.icns"
cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key><string>Swarf</string>
  <key>CFBundleDisplayName</key><string>Swarf</string>
  <key>CFBundleIdentifier</key><string>$BUNDLE_ID</string>
  <key>CFBundleVersion</key><string>$VERSION</string>
  <key>CFBundleShortVersionString</key><string>$VERSION</string>
  <key>CFBundleExecutable</key><string>swarf</string>
  <key>CFBundleIconFile</key><string>Swarf</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>LSMinimumSystemVersion</key><string>11.0</string>
  <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST
echo "Built $APP"
