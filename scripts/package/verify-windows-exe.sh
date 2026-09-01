#!/bin/sh
set -eu

EXE=${1:?"usage: $0 <windows-gui.exe> <x64|x86|arm64> [main|helper]"}
ARCH=${2:?"usage: $0 <windows-gui.exe> <x64|x86|arm64> [main|helper]"}
KIND=${3:-main}

case "$ARCH" in
  x64) MACHINE="IMAGE_FILE_MACHINE_AMD64" ;;
  x86) MACHINE="IMAGE_FILE_MACHINE_I386" ;;
  arm64) MACHINE="IMAGE_FILE_MACHINE_ARM64" ;;
  *)
    echo "Unsupported Windows architecture label: $ARCH" >&2
    exit 1
    ;;
esac

case "$KIND" in
  main|helper) ;;
  *)
    echo "Unsupported Windows executable kind: $KIND" >&2
    exit 1
    ;;
esac

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

if [ "$KIND" = "main" ]; then
  RESOURCES=$($LLVM_READOBJ --coff-resources "$EXE")
  printf '%s\n' "$RESOURCES" | grep -q 'Type: ICON (ID 3)' || {
    echo "Windows executable is missing ICON resources: $EXE" >&2
    exit 1
  }
  printf '%s\n' "$RESOURCES" | grep -q 'Type: GROUP_ICON (ID 14)' || {
    echo "Windows executable is missing GROUP_ICON resources: $EXE" >&2
    exit 1
  }
fi

HEADERS=$($LLVM_READOBJ --file-headers "$EXE")
printf '%s\n' "$HEADERS" | grep -q "Machine: $MACHINE" || {
  echo "Windows executable has the wrong architecture for $ARCH: $EXE" >&2
  exit 1
}
printf '%s\n' "$HEADERS" | grep -q 'Subsystem: IMAGE_SUBSYSTEM_WINDOWS_GUI' || {
  echo "Windows executable is not a GUI PE: $EXE" >&2
  exit 1
}

echo "Verified Windows $KIND GUI ($ARCH): $EXE"
