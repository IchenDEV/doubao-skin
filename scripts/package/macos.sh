#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPO_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)
DIST_DIR="$REPO_DIR/dist"
APP_NAME="豆皮"
BUNDLE="$DIST_DIR/$APP_NAME.app"
LEGACY_BUNDLE="$DIST_DIR/Doubao Skin.app"
EXECUTABLE_NAME="doubao-skin-app"
AGENT_EXECUTABLE_NAME="doubao-skin-agent"
PACKAGE_NAME="doubao-skin-desktop"
APP_VERSION=${APP_VERSION:-$(awk -F '"' '/^version =/ { print $2; exit }' "$REPO_DIR/Cargo.toml")}
CODESIGN_IDENTITY=${CODESIGN_IDENTITY:--}
CODESIGN_KEYCHAIN=${CODESIGN_KEYCHAIN:-}
BUILD_MODE=${1:-host}
DEPLOY_TARGET=${MACOSX_DEPLOYMENT_TARGET:-12.0}
DEFAULT_BUNDLED_THEMES="doubao-snack-giggle doubao-dessert-giggle gallery-whale-maid qq-light-blue pure-dark"
BUNDLED_THEMES=${BUNDLED_THEMES:-$DEFAULT_BUNDLED_THEMES}
APP_ICON_DIR="$REPO_DIR/assets/app-icon"
APP_ICON_SOURCE="$APP_ICON_DIR/AppIcon.icon"
APP_ICON_FALLBACK="$APP_ICON_DIR/AppIcon.icns"
APP_ICON_ASSETS_FALLBACK="$APP_ICON_DIR/Assets.car"
DMG_ASSET_DIR="$REPO_DIR/assets/dmg"
DMG_BACKGROUND_1X="$DMG_ASSET_DIR/install-background.png"
DMG_BACKGROUND_2X="$DMG_ASSET_DIR/install-background@2x.png"

case "$BUILD_MODE" in
  host)
    HOST_TARGET=$(rustc -vV | sed -n 's/^host: //p')
    case "$HOST_TARGET" in
      aarch64-apple-darwin) ARCHIVE_LABEL="arm64" ;;
      x86_64-apple-darwin) ARCHIVE_LABEL="x86_64" ;;
      *)
        echo "macOS packaging requires an Apple Darwin Rust host; found $HOST_TARGET" >&2
        exit 1
        ;;
    esac
    TARGETS="$HOST_TARGET"
    ;;
  --universal)
    TARGETS="aarch64-apple-darwin x86_64-apple-darwin"
    ARCHIVE_LABEL="universal"
    ;;
  *)
    echo "usage: $0 [--universal]" >&2
    exit 2
    ;;
esac

for dmg_asset in "$DMG_BACKGROUND_1X" "$DMG_BACKGROUND_2X"; do
  if [ ! -f "$dmg_asset" ]; then
    echo "missing DMG background asset: $dmg_asset" >&2
    exit 1
  fi
done
for dmg_tool in diskutil hdiutil osascript tiffutil; do
  if ! command -v "$dmg_tool" >/dev/null 2>&1; then
    echo "macOS DMG layout requires $dmg_tool" >&2
    exit 1
  fi
done

mkdir -p "$DIST_DIR"
rm -rf "$BUNDLE" "$LEGACY_BUNDLE"
mkdir -p "$BUNDLE/Contents/MacOS" "$BUNDLE/Contents/Resources/licenses"
AGENT_BUNDLE="$BUNDLE/Contents/Library/LoginItems/豆皮后台服务.app"
mkdir -p "$AGENT_BUNDLE/Contents/MacOS"

for build_target in $TARGETS; do
  rustup target add "$build_target"
  if [ "$BUILD_MODE" = "host" ]; then
    MACOSX_DEPLOYMENT_TARGET="$DEPLOY_TARGET" cargo build \
      --manifest-path "$REPO_DIR/Cargo.toml" \
      --locked \
      --release \
      --package "$PACKAGE_NAME"
  else
    MACOSX_DEPLOYMENT_TARGET="$DEPLOY_TARGET" cargo build \
      --manifest-path "$REPO_DIR/Cargo.toml" \
      --locked \
      --release \
      --package "$PACKAGE_NAME" \
      --target "$build_target"
  fi
done

if [ "$BUILD_MODE" = "--universal" ]; then
  lipo -create \
    "$REPO_DIR/target/aarch64-apple-darwin/release/$EXECUTABLE_NAME" \
    "$REPO_DIR/target/x86_64-apple-darwin/release/$EXECUTABLE_NAME" \
    -output "$BUNDLE/Contents/MacOS/$APP_NAME"
  lipo -create \
    "$REPO_DIR/target/aarch64-apple-darwin/release/$AGENT_EXECUTABLE_NAME" \
    "$REPO_DIR/target/x86_64-apple-darwin/release/$AGENT_EXECUTABLE_NAME" \
    -output "$AGENT_BUNDLE/Contents/MacOS/豆皮后台服务"
else
  cp "$REPO_DIR/target/release/$EXECUTABLE_NAME" "$BUNDLE/Contents/MacOS/$APP_NAME"
  cp "$REPO_DIR/target/release/$AGENT_EXECUTABLE_NAME" \
    "$AGENT_BUNDLE/Contents/MacOS/豆皮后台服务"
fi

cp "$REPO_DIR/apps/desktop/Info.plist" "$BUNDLE/Contents/Info.plist"
cp "$REPO_DIR/apps/desktop/Agent-Info.plist" "$AGENT_BUNDLE/Contents/Info.plist"

ICON_DEVELOPER_DIR=${ICON_DEVELOPER_DIR:-}
if [ -n "$ICON_DEVELOPER_DIR" ] && [ ! -x "$ICON_DEVELOPER_DIR/usr/bin/actool" ]; then
  echo "ICON_DEVELOPER_DIR does not provide actool: $ICON_DEVELOPER_DIR" >&2
  exit 1
fi
if [ -z "$ICON_DEVELOPER_DIR" ]; then
  selected_developer_dir=$(xcode-select -p 2>/dev/null || true)
  if [ -x "$selected_developer_dir/usr/bin/actool" ]; then
    ICON_DEVELOPER_DIR="$selected_developer_dir"
  elif [ -x "/Applications/Xcode.app/Contents/Developer/usr/bin/actool" ]; then
    ICON_DEVELOPER_DIR="/Applications/Xcode.app/Contents/Developer"
  elif [ -x "/Applications/Xcode-beta.app/Contents/Developer/usr/bin/actool" ]; then
    ICON_DEVELOPER_DIR="/Applications/Xcode-beta.app/Contents/Developer"
  fi
fi

ICON_PARTIAL_PLIST="$DIST_DIR/.AppIcon-partial.plist"
actool_ok=false
if [ -n "$ICON_DEVELOPER_DIR" ] && DEVELOPER_DIR="$ICON_DEVELOPER_DIR" xcrun actool \
  "$APP_ICON_SOURCE" \
  --compile "$BUNDLE/Contents/Resources" \
  --platform macosx \
  --minimum-deployment-target "$DEPLOY_TARGET" \
  --app-icon AppIcon \
  --output-partial-info-plist "$ICON_PARTIAL_PLIST" \
  --warnings \
  --notices; then
  rm -f "$ICON_PARTIAL_PLIST"
  if [ -f "$BUNDLE/Contents/Resources/AppIcon.icns" ] && [ -f "$BUNDLE/Contents/Resources/Assets.car" ]; then
    actool_ok=true
    echo "Compiled adaptive app icon from AppIcon.icon"
  else
    echo "actool returned 0 but icon files are missing; falling back to precompiled assets"
  fi
else
  rm -f "$ICON_PARTIAL_PLIST"
fi
if [ "$actool_ok" = false ]; then
  if [ ! -f "$APP_ICON_FALLBACK" ] || [ ! -f "$APP_ICON_ASSETS_FALLBACK" ]; then
    echo "building the adaptive app icon requires Icon Composer actool or checked-in AppIcon.icns and Assets.car" >&2
    exit 1
  fi
  cp "$APP_ICON_FALLBACK" "$BUNDLE/Contents/Resources/AppIcon.icns"
  cp "$APP_ICON_ASSETS_FALLBACK" "$BUNDLE/Contents/Resources/Assets.car"
  echo "Used precompiled adaptive app icon resources"
fi
THEMES_DEST="$BUNDLE/Contents/Resources/themes"
mkdir -p "$THEMES_DEST"
if [ "${BUNDLE_ALL_THEMES:-0}" = "1" ]; then
  cp -R "$REPO_DIR/themes/." "$THEMES_DEST"
  echo "Bundled all themes"
else
  for theme_id in $BUNDLED_THEMES; do
    theme_source="$REPO_DIR/themes/$theme_id"
    if [ ! -d "$theme_source" ]; then
      echo "bundled theme does not exist: $theme_id" >&2
      exit 1
    fi
    cp -R "$theme_source" "$THEMES_DEST/$theme_id"
  done
  echo "Bundled themes: $BUNDLED_THEMES"
fi
cp "$REPO_DIR/LICENSE" "$BUNDLE/Contents/Resources/licenses/MIT.txt"
cp "$REPO_DIR/LICENSES/GPL-3.0-or-later.txt" "$BUNDLE/Contents/Resources/licenses/GPL-3.0-or-later.txt"
cp "$REPO_DIR/THIRD_PARTY_NOTICES.md" "$BUNDLE/Contents/Resources/licenses/THIRD_PARTY_NOTICES.md"
/usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString $APP_VERSION" "$BUNDLE/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Set :CFBundleVersion ${GITHUB_RUN_NUMBER:-1}" "$BUNDLE/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Set :CFBundleShortVersionString $APP_VERSION" "$AGENT_BUNDLE/Contents/Info.plist"
/usr/libexec/PlistBuddy -c "Set :CFBundleVersion ${GITHUB_RUN_NUMBER:-1}" "$AGENT_BUNDLE/Contents/Info.plist"

plutil -lint "$BUNDLE/Contents/Info.plist" "$AGENT_BUNDLE/Contents/Info.plist"
agent_bundle_id=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$AGENT_BUNDLE/Contents/Info.plist")
agent_ui_element=$(/usr/libexec/PlistBuddy -c 'Print :LSUIElement' "$AGENT_BUNDLE/Contents/Info.plist")
main_version=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$BUNDLE/Contents/Info.plist")
agent_version=$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$AGENT_BUNDLE/Contents/Info.plist")
main_minimum=$(/usr/libexec/PlistBuddy -c 'Print :LSMinimumSystemVersion' "$BUNDLE/Contents/Info.plist")
agent_minimum=$(/usr/libexec/PlistBuddy -c 'Print :LSMinimumSystemVersion' "$AGENT_BUNDLE/Contents/Info.plist")
if [ "$agent_bundle_id" != "dev.ichen.doubao-skin.agent" ] || [ "$agent_ui_element" != "true" ]; then
  echo "invalid bundled login item metadata" >&2
  exit 1
fi
if [ "$main_version" != "$agent_version" ] || [ "$main_minimum" != "$agent_minimum" ]; then
  echo "main app and login item versions must match" >&2
  exit 1
fi
if [ "$BUILD_MODE" = "--universal" ]; then
  agent_archs=$(lipo -archs "$AGENT_BUNDLE/Contents/MacOS/豆皮后台服务")
  case "$agent_archs" in
    *arm64*x86_64*|*x86_64*arm64*) ;;
    *) echo "universal login item is missing an architecture: $agent_archs" >&2; exit 1 ;;
  esac
fi

if [ -n "$CODESIGN_KEYCHAIN" ]; then
  codesign --force --options runtime --timestamp=none --keychain "$CODESIGN_KEYCHAIN" --sign "$CODESIGN_IDENTITY" "$AGENT_BUNDLE"
  codesign --force --options runtime --timestamp=none --keychain "$CODESIGN_KEYCHAIN" --sign "$CODESIGN_IDENTITY" "$BUNDLE"
else
  codesign --force --options runtime --timestamp=none --sign "$CODESIGN_IDENTITY" "$AGENT_BUNDLE"
  codesign --force --options runtime --timestamp=none --sign "$CODESIGN_IDENTITY" "$BUNDLE"
fi
codesign --verify --strict "$AGENT_BUNDLE"
codesign --verify --deep --strict "$BUNDLE"

NOTARY_VALUES="${APPLE_ID:-}${APPLE_TEAM_ID:-}${APPLE_APP_PASSWORD:-}"
if [ -n "$NOTARY_VALUES" ]; then
  if [ -z "${APPLE_ID:-}" ] || [ -z "${APPLE_TEAM_ID:-}" ] || [ -z "${APPLE_APP_PASSWORD:-}" ]; then
    echo "APPLE_ID, APPLE_TEAM_ID and APPLE_APP_PASSWORD must be set together" >&2
    exit 1
  fi
  if [ "$CODESIGN_IDENTITY" = "-" ]; then
    echo "notarization requires a Developer ID Application signing identity" >&2
    exit 1
  fi
  NOTARY_ARCHIVE="$DIST_DIR/Doubao-Skin-notarization.zip"
  rm -f "$NOTARY_ARCHIVE"
  ditto -c -k --keepParent "$BUNDLE" "$NOTARY_ARCHIVE"
  xcrun notarytool submit "$NOTARY_ARCHIVE" \
    --apple-id "$APPLE_ID" \
    --team-id "$APPLE_TEAM_ID" \
    --password "$APPLE_APP_PASSWORD" \
    --wait
  xcrun stapler staple "$BUNDLE"
  rm -f "$NOTARY_ARCHIVE"
fi

ARCHIVE_BASENAME="Doubao-Skin-macOS-$ARCHIVE_LABEL.zip"
ARCHIVE="$DIST_DIR/$ARCHIVE_BASENAME"
rm -f "$ARCHIVE" "$ARCHIVE.sha256"
ditto -c -k --sequesterRsrc --keepParent "$BUNDLE" "$ARCHIVE"
(
  cd "$DIST_DIR"
  shasum -a 256 "$ARCHIVE_BASENAME" > "$ARCHIVE_BASENAME.sha256"
)

DMG_BASENAME="Doubao-Skin-macOS-$ARCHIVE_LABEL.dmg"
DMG="$DIST_DIR/$DMG_BASENAME"
DMG_TEMP="$DIST_DIR/.Doubao-Skin-macOS-$ARCHIVE_LABEL.tmp.dmg"
DMG_RW="$DIST_DIR/.Doubao-Skin-macOS-$ARCHIVE_LABEL.rw.dmg"
DMG_STAGING=$(mktemp -d "$DIST_DIR/.dmg-staging.XXXXXX")
DMG_BUILD_VOLUME_NAME="$APP_NAME-DMG-$ARCHIVE_LABEL-$$"
DMG_MOUNT="/Volumes/$DMG_BUILD_VOLUME_NAME"
DMG_DEVICE=""
DMG_ATTACHED=false
cleanup_dmg() {
  if [ "${DMG_ATTACHED:-false}" = true ]; then
    if [ -n "${DMG_DEVICE:-}" ]; then
      hdiutil detach "$DMG_DEVICE" -quiet 2>/dev/null || true
    elif [ -n "${DMG_MOUNT:-}" ]; then
      hdiutil detach "$DMG_MOUNT" -quiet 2>/dev/null || true
    fi
    DMG_ATTACHED=false
  fi
  if [ -n "${DMG_STAGING:-}" ] && [ -d "$DMG_STAGING" ]; then
    rm -rf "$DMG_STAGING"
  fi
  if [ -n "${DMG_RW:-}" ] && [ -f "$DMG_RW" ]; then
    rm -f "$DMG_RW"
  fi
  if [ -n "${DMG_TEMP:-}" ] && [ -f "$DMG_TEMP" ]; then
    rm -f "$DMG_TEMP"
  fi
}
trap cleanup_dmg EXIT HUP INT TERM
rm -f "$DMG" "$DMG.sha256" "$DMG_TEMP" "$DMG_RW"
cp -R "$BUNDLE" "$DMG_STAGING/$APP_NAME.app"
ln -s /Applications "$DMG_STAGING/Applications"
mkdir -p "$DMG_STAGING/.background"
tiffutil -cathidpicheck \
  "$DMG_BACKGROUND_1X" \
  "$DMG_BACKGROUND_2X" \
  -out "$DMG_STAGING/.background/install-background.tiff"
hdiutil create \
  -volname "$DMG_BUILD_VOLUME_NAME" \
  -srcfolder "$DMG_STAGING" \
  -format UDRW \
  -ov \
  "$DMG_RW"
if [ -e "$DMG_MOUNT" ]; then
  echo "temporary DMG volume path already exists: $DMG_MOUNT" >&2
  exit 1
fi
DMG_ATTACH_OUTPUT=$(hdiutil attach "$DMG_RW" -noverify -readwrite)
printf '%s\n' "$DMG_ATTACH_OUTPUT"
DMG_ATTACHED=true
DMG_DEVICE=$(printf '%s\n' "$DMG_ATTACH_OUTPUT" | awk '$1 ~ /^\/dev\// && $NF ~ /^\/Volumes\// { print $1; exit }')
if [ -z "$DMG_DEVICE" ] || [ ! -d "$DMG_MOUNT" ]; then
  echo "could not resolve temporary DMG device or mount: $DMG_BUILD_VOLUME_NAME" >&2
  exit 1
fi

osascript - "$DMG_BUILD_VOLUME_NAME" "$APP_NAME.app" <<'APPLESCRIPT'
on run argv
  set volumeName to item 1 of argv
  set appFileName to item 2 of argv

  tell application "Finder"
    tell disk volumeName
      open
      set installerWindow to container window
      set current view of installerWindow to icon view
      set toolbar visible of installerWindow to false
      set statusbar visible of installerWindow to false
      set pathbar visible of installerWindow to false
      set bounds of installerWindow to {100, 100, 760, 528}

      set iconOptions to icon view options of installerWindow
      set arrangement of iconOptions to not arranged
      set icon size of iconOptions to 120
      set text size of iconOptions to 14
      set label position of iconOptions to bottom
      set shows item info of iconOptions to false
      set shows icon preview of iconOptions to false
      set background picture of iconOptions to file ".background:install-background.tiff"

      set position of item appFileName to {170, 220}
      set position of item "Applications" to {490, 220}
      set extension hidden of item appFileName to true
      update without registering applications
      delay 2
      close
    end tell
  end tell
end run
APPLESCRIPT

sync
test -f "$DMG_MOUNT/.DS_Store"
test -f "$DMG_MOUNT/.background/install-background.tiff"
test -d "$DMG_MOUNT/$APP_NAME.app"
test "$(readlink "$DMG_MOUNT/Applications")" = "/Applications"
diskutil rename "$DMG_DEVICE" "$APP_NAME" >/dev/null
hdiutil detach "$DMG_DEVICE" -quiet
DMG_ATTACHED=false
DMG_MOUNT=""
DMG_DEVICE=""

hdiutil convert \
  "$DMG_RW" \
  -format UDZO \
  -imagekey zlib-level=9 \
  -ov \
  -o "$DMG_TEMP"
hdiutil verify "$DMG_TEMP"

if [ -n "$NOTARY_VALUES" ]; then
  xcrun notarytool submit "$DMG_TEMP" \
    --apple-id "$APPLE_ID" \
    --team-id "$APPLE_TEAM_ID" \
    --password "$APPLE_APP_PASSWORD" \
    --wait
  xcrun stapler staple "$DMG_TEMP"
  xcrun stapler validate "$DMG_TEMP"
  hdiutil verify "$DMG_TEMP"
fi

mv "$DMG_TEMP" "$DMG"
DMG_TEMP=""
cleanup_dmg
DMG_STAGING=""
DMG_MOUNT=""
DMG_RW=""
trap - EXIT HUP INT TERM
(
  cd "$DIST_DIR"
  shasum -a 256 "$DMG_BASENAME" > "$DMG_BASENAME.sha256"
)

echo "Built $ARCHIVE"
echo "Checksum $ARCHIVE.sha256"
echo "Built $DMG"
echo "Checksum $DMG.sha256"
