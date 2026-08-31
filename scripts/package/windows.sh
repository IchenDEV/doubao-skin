#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPO_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)
DIST_DIR="$REPO_DIR/dist"
EXECUTABLE_NAME="doubao-skin-app"
PACKAGED_EXECUTABLE_NAME="doubao-skin"
PACKAGE_NAME="doubao-skin-desktop"
DEFAULT_BUNDLED_THEMES="doubao-snack-giggle doubao-dessert-giggle gallery-whale-maid qq-light-blue pure-dark"
BUNDLED_THEMES=${BUNDLED_THEMES:-$DEFAULT_BUNDLED_THEMES}

TARGET=${1:?"usage: $0 <target-triple>  (x86_64-pc-windows-msvc | i686-pc-windows-msvc | aarch64-pc-windows-msvc)"}

case "$TARGET" in
  x86_64-pc-windows-msvc)   LABEL="x64" ;;
  i686-pc-windows-msvc)
    LABEL="x86"
    # psm ships a 32-bit assembly object without a SafeSEH table. The object is
    # still linked with DEP and ASLR; only the incompatible SafeSEH check is off.
    RUSTFLAGS="${RUSTFLAGS:-} -C link-arg=/SAFESEH:NO"
    export RUSTFLAGS
    ;;
  aarch64-pc-windows-msvc)
    LABEL="arm64"
    ;;
  *)
    echo "Unsupported Windows target: $TARGET" >&2
    echo "Supported: x86_64-pc-windows-msvc, i686-pc-windows-msvc, aarch64-pc-windows-msvc" >&2
    exit 1
    ;;
esac

mkdir -p "$DIST_DIR"

HOST=$(rustc -vV | awk '/^host:/ { print $2 }')
case "$HOST" in
  *-pc-windows-msvc) ;;
  *)
    echo "Windows desktop packages must be built on a Windows MSVC host (current host: $HOST)" >&2
    exit 1
    ;;
esac

rustup component add llvm-tools-preview
rustup target add "$TARGET"
cargo build \
  --manifest-path "$REPO_DIR/Cargo.toml" \
  --locked \
  --release \
  --package "$PACKAGE_NAME" \
  --target "$TARGET"

"$SCRIPT_DIR/verify-windows-exe.sh" \
  "$REPO_DIR/target/$TARGET/release/$EXECUTABLE_NAME.exe"

STAGING="$DIST_DIR/Doubao-Skin-Windows-$LABEL"
rm -rf "$STAGING"
mkdir -p "$STAGING/themes" "$STAGING/licenses"

cp "$REPO_DIR/target/$TARGET/release/$EXECUTABLE_NAME.exe" "$STAGING/$PACKAGED_EXECUTABLE_NAME.exe"
"$SCRIPT_DIR/verify-windows-exe.sh" "$STAGING/$PACKAGED_EXECUTABLE_NAME.exe"

if [ "${BUNDLE_ALL_THEMES:-0}" = "1" ]; then
  cp -R "$REPO_DIR/themes/." "$STAGING/themes"
  echo "Bundled all themes"
else
  for theme_id in $BUNDLED_THEMES; do
    theme_source="$REPO_DIR/themes/$theme_id"
    if [ ! -d "$theme_source" ]; then
      echo "bundled theme does not exist: $theme_id" >&2
      exit 1
    fi
    cp -R "$theme_source" "$STAGING/themes/$theme_id"
  done
  echo "Bundled themes: $BUNDLED_THEMES"
fi

cp "$REPO_DIR/LICENSE" "$STAGING/licenses/MIT.txt"
cp "$REPO_DIR/LICENSES/GPL-3.0-or-later.txt" "$STAGING/licenses/GPL-3.0-or-later.txt"
cp "$REPO_DIR/THIRD_PARTY_NOTICES.md" "$STAGING/licenses/THIRD_PARTY_NOTICES.md"

top_level_executables=$(find "$STAGING" -maxdepth 1 -type f -name '*.exe' | wc -l | tr -d ' ')
if [ "$top_level_executables" != "1" ]; then
  echo "Windows package must contain exactly one top-level executable" >&2
  exit 1
fi

ARCHIVE_BASENAME="Doubao-Skin-Windows-$LABEL.zip"
ARCHIVE="$DIST_DIR/$ARCHIVE_BASENAME"
rm -f "$ARCHIVE" "$ARCHIVE.sha256"

(
  cd "$DIST_DIR"
  if command -v 7z >/dev/null 2>&1; then
    7z a -tzip "$ARCHIVE_BASENAME" "Doubao-Skin-Windows-$LABEL/"
  elif command -v zip >/dev/null 2>&1; then
    zip -r "$ARCHIVE_BASENAME" "Doubao-Skin-Windows-$LABEL/"
  elif command -v powershell.exe >/dev/null 2>&1; then
    powershell.exe -NoProfile -Command \
      "Compress-Archive -Force -Path 'Doubao-Skin-Windows-$LABEL' -DestinationPath '$ARCHIVE_BASENAME'"
  else
    echo "No zip tool available (tried 7z, zip, powershell)" >&2
    exit 1
  fi
)

sha256_file() {
  (
    cd "$(dirname "$1")"
    base=$(basename "$1")
    if command -v shasum >/dev/null 2>&1; then
      shasum -a 256 "$base" > "$base.sha256"
    elif command -v sha256sum >/dev/null 2>&1; then
      sha256sum "$base" > "$base.sha256"
    elif command -v certutil.exe >/dev/null 2>&1; then
      hash=$(certutil.exe -hashfile "$base" SHA256 | sed -n '2p' | tr -d ' \r')
      printf '%s  %s\n' "$hash" "$base" > "$base.sha256"
    else
      echo "No SHA-256 tool available" >&2
      exit 1
    fi
  )
}
sha256_file "$ARCHIVE"

echo "Built $ARCHIVE"
echo "Checksum $ARCHIVE.sha256"
