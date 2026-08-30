#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/../.." && pwd)
cd "$REPO_ROOT"

failures=0

reject() {
  description=$1
  pattern=$2
  shift 2
  matches=$(rg -n --glob '*.rs' -e "$pattern" "$@" || true)
  if [ -n "$matches" ]; then
    printf 'portability: %s\n%s\n' "$description" "$matches" >&2
    failures=1
  fi
}

reject_outside_adapters() {
  description=$1
  pattern=$2
  allowed=$3
  shift 3
  matches=$(rg -n --glob '*.rs' -e "$pattern" "$@" || true)
  matches=$(printf '%s\n' "$matches" | grep -Ev "$allowed" || true)
  if [ -n "$matches" ]; then
    printf 'portability: %s\n%s\n' "$description" "$matches" >&2
    failures=1
  fi
}

source_roots=(crates apps)

reject "use the platform random provider instead of Unix device files" \
  '/dev/(u?random|null)' "${source_roots[@]}"
reject "never fall back to a relative persistence path" \
  'PathBuf::from\("target-app"\)' "${source_roots[@]}"
reject "runtime HTTP must not depend on an external curl process" \
  'Command::new\("curl"\)' "${source_roots[@]}"

adapter_files='^(crates/skin-core/src/live/platform\.rs|crates/skin-core/src/build/macos\.rs):'
reject_outside_adapters \
  "platform directory environment variables belong only in explicit adapters" \
  'std::env::var(_os)?\("(HOME|USERPROFILE|LOCALAPPDATA|APPDATA|ProgramFiles|ProgramFiles\(x86\))"\)' \
  "$adapter_files" "${source_roots[@]}"
reject_outside_adapters \
  "installed-application paths belong only in explicit adapters" \
  '"(/Applications/|[A-Za-z]:[\\/]Program Files( \(x86\))?[\\/]|%(LOCALAPPDATA|APPDATA|ProgramFiles)%[\\/])' \
  "$adapter_files" "${source_roots[@]}"
reject_outside_adapters \
  "operating-system process commands belong only in explicit adapters" \
  'Command::new\("(open|osascript|pkill|pgrep|kill|killall|taskkill|tasklist|cmd|cmd\.exe|powershell|powershell\.exe|reg|reg\.exe|explorer|launchctl|defaults|codesign|xattr|touch|cp|security|hdiutil)"\)' \
  "$adapter_files" "${source_roots[@]}"

case_collisions=$(find crates apps scripts \
  -type f -print | awk '{ key=tolower($0); if (seen[key] && seen[key] != $0) print seen[key] " <-> " $0; seen[key]=$0 }')
if [ -n "$case_collisions" ]; then
  printf 'portability: case-insensitive filesystems would collide\n%s\n' \
    "$case_collisions" >&2
  failures=1
fi

if [ "$failures" -ne 0 ]; then
  exit 1
fi

echo "portability: runtime platform boundaries passed"
