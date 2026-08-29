#!/bin/sh
set -eu

REPO="IchenDEV/doubao-skin"
BINARY_NAME="doubao-theme"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

main() {
  check_platform
  version=$(resolve_version)
  asset="doubao-theme-macOS-universal.tar.gz"
  url="https://github.com/$REPO/releases/download/$version/$asset"
  checksum_url="${url}.sha256"

  tmp=$(mktemp -d)
  cleanup() { rm -rf "$tmp"; }
  trap cleanup EXIT HUP INT TERM

  printf "Downloading %s %s...\n" "$BINARY_NAME" "$version"
  curl -fsSL "$url" -o "$tmp/$asset"
  curl -fsSL "$checksum_url" -o "$tmp/$asset.sha256"

  printf "Verifying checksum...\n"
  (cd "$tmp" && shasum -a 256 -c "$asset.sha256")

  tar -xzf "$tmp/$asset" -C "$tmp"

  if [ ! -d "$INSTALL_DIR" ]; then
    mkdir -p "$INSTALL_DIR"
  fi

  if [ -w "$INSTALL_DIR" ]; then
    cp "$tmp/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
    chmod 755 "$INSTALL_DIR/$BINARY_NAME"
  else
    printf "Installing to %s requires elevated permissions.\n" "$INSTALL_DIR"
    sudo cp "$tmp/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
    sudo chmod 755 "$INSTALL_DIR/$BINARY_NAME"
  fi

  printf "Installed %s to %s/%s\n" "$version" "$INSTALL_DIR" "$BINARY_NAME"
  "$INSTALL_DIR/$BINARY_NAME" --version
}

check_platform() {
  os=$(uname -s)
  if [ "$os" != "Darwin" ]; then
    printf "doubao-theme is currently macOS-only (detected: %s)\n" "$os" >&2
    exit 1
  fi
}

resolve_version() {
  if [ -n "${VERSION:-}" ]; then
    printf '%s' "$VERSION"
    return
  fi
  release_url="https://api.github.com/repos/$REPO/releases/latest"
  tag=$(curl -fsSL "$release_url" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -1)
  if [ -z "$tag" ]; then
    printf "Could not determine latest release from %s\n" "$release_url" >&2
    exit 1
  fi
  printf '%s' "$tag"
}

main
