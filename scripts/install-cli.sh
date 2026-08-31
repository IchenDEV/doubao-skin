#!/bin/sh
set -eu

REPO="IchenDEV/doubao-skin"
BINARY_NAME="doubao-skin"
INSTALL_DIR=${INSTALL_DIR:-/usr/local/bin}

platform_asset() {
  os=$(uname -s)
  machine=$(uname -m)
  case "$os" in
    Darwin) printf '%s' "doubao-skin-cli-macOS-universal.tar.gz" ;;
    Linux)
      case "$machine" in
        x86_64|amd64) printf '%s' "doubao-skin-cli-Linux-x64.tar.gz" ;;
        aarch64|arm64) printf '%s' "doubao-skin-cli-Linux-arm64.tar.gz" ;;
        *) echo "Unsupported Linux architecture: $machine" >&2; exit 1 ;;
      esac
      ;;
    *)
      echo "This installer supports macOS and Linux. On Windows, install the CLI with Scoop." >&2
      exit 1
      ;;
  esac
}

download_base() {
  if [ -n "${VERSION:-}" ]; then
    case "$VERSION" in v*) tag=$VERSION ;; *) tag="v$VERSION" ;; esac
    printf 'https://github.com/%s/releases/download/%s' "$REPO" "$tag"
  else
    printf 'https://github.com/%s/releases/latest/download' "$REPO"
  fi
}

verify_checksum() {
  directory=$1
  checksum_file=$2
  (
    cd "$directory"
    if command -v shasum >/dev/null 2>&1; then
      shasum -a 256 -c "$checksum_file"
    elif command -v sha256sum >/dev/null 2>&1; then
      sha256sum -c "$checksum_file"
    else
      echo "No SHA-256 tool available" >&2
      exit 1
    fi
  )
}

main() {
  asset=$(platform_asset)
  base=$(download_base)
  temporary_directory=$(mktemp -d)
  trap 'rm -rf "$temporary_directory"' EXIT HUP INT TERM

  echo "Downloading $asset..."
  curl -fsSL "$base/$asset" -o "$temporary_directory/$asset"
  curl -fsSL "$base/$asset.sha256" -o "$temporary_directory/$asset.sha256"
  verify_checksum "$temporary_directory" "$asset.sha256"
  tar -xzf "$temporary_directory/$asset" -C "$temporary_directory"

  if [ ! -d "$INSTALL_DIR" ]; then
    mkdir -p "$INSTALL_DIR"
  fi
  if [ -w "$INSTALL_DIR" ]; then
    cp "$temporary_directory/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
    chmod 755 "$INSTALL_DIR/$BINARY_NAME"
  else
    echo "Installing to $INSTALL_DIR requires elevated permissions."
    sudo cp "$temporary_directory/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
    sudo chmod 755 "$INSTALL_DIR/$BINARY_NAME"
  fi

  "$INSTALL_DIR/$BINARY_NAME" --version
}

main
