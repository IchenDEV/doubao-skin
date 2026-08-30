#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPO_DIR=$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)
DIST_DIR="$REPO_DIR/dist"
BINARY_NAME="doubao-skin"
PACKAGE_NAME="skin-core"
BUILD_MODE=${1:---host}

mkdir -p "$DIST_DIR"

build_windows() {
  target=$1
  case "$target" in
    x86_64-pc-windows-msvc) label="x64" ;;
    i686-pc-windows-msvc) label="x86" ;;
    aarch64-pc-windows-msvc) label="arm64" ;;
    *) return 1 ;;
  esac

  host=$(rustc -vV | sed -n 's/^host: //p')
  case "$host" in
    *-pc-windows-msvc) ;;
    *)
      echo "Windows CLI packages must be built on a Windows MSVC host (current host: $host)" >&2
      exit 1
      ;;
  esac

  rustup target add "$target"
  cargo build \
    --manifest-path "$REPO_DIR/Cargo.toml" \
    --locked \
    --release \
    --package "$PACKAGE_NAME" \
    --bin "$BINARY_NAME" \
    --target "$target"

  staging=$(mktemp -d "$DIST_DIR/.cli-staging.XXXXXX")
  trap 'rm -rf "$staging"' EXIT HUP INT TERM
  cp "$REPO_DIR/target/$target/release/$BINARY_NAME.exe" "$staging/$BINARY_NAME.exe"
  cp "$REPO_DIR/LICENSE" "$staging/LICENSE"

  archive_basename="doubao-skin-cli-Windows-$label.zip"
  archive="$DIST_DIR/$archive_basename"
  rm -f "$archive" "$archive.sha256"
  (
    cd "$staging"
    if command -v 7z >/dev/null 2>&1; then
      7z a -tzip "$archive" "$BINARY_NAME.exe" LICENSE
    elif command -v zip >/dev/null 2>&1; then
      zip "$archive" "$BINARY_NAME.exe" LICENSE
    else
      echo "No zip tool available" >&2
      exit 1
    fi
  )
  write_checksum "$archive"
  rm -rf "$staging"
  trap - EXIT HUP INT TERM
  report "$archive"
}

build_universal_macos() {
  for target in aarch64-apple-darwin x86_64-apple-darwin; do
    rustup target add "$target"
    MACOSX_DEPLOYMENT_TARGET=${MACOSX_DEPLOYMENT_TARGET:-12.0} cargo build \
      --manifest-path "$REPO_DIR/Cargo.toml" \
      --locked \
      --release \
      --package "$PACKAGE_NAME" \
      --bin "$BINARY_NAME" \
      --target "$target"
  done

  staging=$(mktemp -d "$DIST_DIR/.cli-staging.XXXXXX")
  trap 'rm -rf "$staging"' EXIT HUP INT TERM
  lipo -create \
    "$REPO_DIR/target/aarch64-apple-darwin/release/$BINARY_NAME" \
    "$REPO_DIR/target/x86_64-apple-darwin/release/$BINARY_NAME" \
    -output "$staging/$BINARY_NAME"
  chmod 755 "$staging/$BINARY_NAME"
  cp "$REPO_DIR/LICENSE" "$staging/LICENSE"

  archive="$DIST_DIR/doubao-skin-cli-macOS-universal.tar.gz"
  rm -f "$archive" "$archive.sha256"
  tar -czf "$archive" -C "$staging" "$BINARY_NAME" LICENSE
  write_checksum "$archive"
  "$staging/$BINARY_NAME" --version
  rm -rf "$staging"
  trap - EXIT HUP INT TERM
  report "$archive"
}

build_host() {
  host=$(rustc -vV | sed -n 's/^host: //p')
  case "$host" in
    x86_64-unknown-linux-gnu) platform="Linux"; label="x64" ;;
    aarch64-unknown-linux-gnu) platform="Linux"; label="arm64" ;;
    x86_64-apple-darwin) platform="macOS"; label="x86_64" ;;
    aarch64-apple-darwin) platform="macOS"; label="arm64" ;;
    *)
      echo "Unsupported CLI host: $host" >&2
      exit 1
      ;;
  esac

  cargo build \
    --manifest-path "$REPO_DIR/Cargo.toml" \
    --locked \
    --release \
    --package "$PACKAGE_NAME" \
    --bin "$BINARY_NAME"

  staging=$(mktemp -d "$DIST_DIR/.cli-staging.XXXXXX")
  trap 'rm -rf "$staging"' EXIT HUP INT TERM
  cp "$REPO_DIR/target/release/$BINARY_NAME" "$staging/$BINARY_NAME"
  chmod 755 "$staging/$BINARY_NAME"
  cp "$REPO_DIR/LICENSE" "$staging/LICENSE"

  archive="$DIST_DIR/doubao-skin-cli-$platform-$label.tar.gz"
  rm -f "$archive" "$archive.sha256"
  tar -czf "$archive" -C "$staging" "$BINARY_NAME" LICENSE
  write_checksum "$archive"
  "$staging/$BINARY_NAME" --version
  rm -rf "$staging"
  trap - EXIT HUP INT TERM
  report "$archive"
}

write_checksum() {
  file=$1
  (
    cd "$(dirname "$file")"
    base=$(basename "$file")
    if command -v shasum >/dev/null 2>&1; then
      shasum -a 256 "$base" > "$base.sha256"
    elif command -v sha256sum >/dev/null 2>&1; then
      sha256sum "$base" > "$base.sha256"
    else
      echo "No SHA-256 tool available" >&2
      exit 1
    fi
  )
}

report() {
  echo "Built $1"
  echo "Checksum $1.sha256"
}

case "$BUILD_MODE" in
  --host) build_host ;;
  --universal-macos) build_universal_macos ;;
  *-pc-windows-msvc) build_windows "$BUILD_MODE" ;;
  *)
    echo "usage: $0 [--host | --universal-macos | <windows-target>]" >&2
    exit 2
    ;;
esac
