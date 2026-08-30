#!/bin/sh
set -eu

EXE=${1:?"usage: $0 <windows-gui.exe>"}

if [ ! -f "$EXE" ]; then
  echo "Windows executable not found: $EXE" >&2
  exit 1
fi

if [ -z "${LLVM_READOBJ:-}" ]; then
  HOST=$(rustc -vV | awk '/^host:/ { print $2 }')
  LLVM_BIN=$(rustc --print sysroot)/lib/rustlib/$HOST/bin
  if [ -x "$LLVM_BIN/llvm-readobj" ]; then
    LLVM_READOBJ="$LLVM_BIN/llvm-readobj"
  elif [ -x "$LLVM_BIN/llvm-readobj.exe" ]; then
    LLVM_READOBJ="$LLVM_BIN/llvm-readobj.exe"
  elif command -v llvm-readobj >/dev/null 2>&1; then
    LLVM_READOBJ=$(command -v llvm-readobj)
  fi
fi
if [ -z "${LLVM_READOBJ:-}" ] || [ ! -x "$LLVM_READOBJ" ]; then
  echo "llvm-readobj not found in the active Rust toolchain" >&2
  exit 1
fi

RESOURCES=$($LLVM_READOBJ --coff-resources "$EXE")
printf '%s\n' "$RESOURCES" | grep -q 'Type: ICON (ID 3)' || {
  echo "Windows executable is missing ICON resources: $EXE" >&2
  exit 1
}
printf '%s\n' "$RESOURCES" | grep -q 'Type: GROUP_ICON (ID 14)' || {
  echo "Windows executable is missing GROUP_ICON resources: $EXE" >&2
  exit 1
}

HEADERS=$($LLVM_READOBJ --file-headers "$EXE")
printf '%s\n' "$HEADERS" | grep -q 'Subsystem: IMAGE_SUBSYSTEM_WINDOWS_GUI' || {
  echo "Windows executable is not a GUI PE: $EXE" >&2
  exit 1
}

echo "Verified Windows GUI resources: $EXE"
