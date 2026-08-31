#!/bin/sh
set -eu

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

command_name=${1:-}
if [ -n "$command_name" ]; then
  shift
fi

case "$command_name" in
  desktop-macos)
    exec "$SCRIPT_DIR/package/macos.sh" "$@"
    ;;
  desktop-windows)
    exec "$SCRIPT_DIR/package/windows.sh" "$@"
    ;;
  cli)
    exec "$SCRIPT_DIR/package/cli.sh" "$@"
    ;;
  scoop)
    exec node "$SCRIPT_DIR/package/generate-scoop-manifest.mjs" "$@"
    ;;
  verify-macos)
    exec "$SCRIPT_DIR/package/verify-macos-signature.sh" "$@"
    ;;
  *)
    echo "usage: $0 <desktop-macos|desktop-windows|cli|scoop|verify-macos> [arguments...]" >&2
    exit 2
    ;;
esac
