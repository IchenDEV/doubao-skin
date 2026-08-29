#!/bin/sh
set -eu

if [ "$#" -ne 2 ]; then
  echo "usage: $0 <app-bundle> <expected-certificate-sha256>" >&2
  exit 2
fi

APP_BUNDLE=$1
EXPECTED_INPUT=$2

if [ ! -d "$APP_BUNDLE" ]; then
  echo "app bundle does not exist: $APP_BUNDLE" >&2
  exit 1
fi

EXPECTED=$(printf '%s' "$EXPECTED_INPUT" | tr -d ':' | tr '[:lower:]' '[:upper:]')
case "$EXPECTED" in
  ''|*[!0-9A-F]*)
    echo "expected certificate SHA-256 must be hexadecimal" >&2
    exit 2
    ;;
esac
if [ "${#EXPECTED}" -ne 64 ]; then
  echo "expected certificate SHA-256 must contain 64 hexadecimal characters" >&2
  exit 2
fi

codesign --verify --deep --strict "$APP_BUNDLE"

VERIFY_DIR=$(mktemp -d "${TMPDIR:-/tmp}/doubao-skin-signature.XXXXXX")
cleanup() {
  rm -rf "$VERIFY_DIR"
}
trap cleanup EXIT HUP INT TERM

CERTIFICATE_PREFIX="$VERIFY_DIR/certificate"
codesign -d --extract-certificates="$CERTIFICATE_PREFIX" "$APP_BUNDLE" >/dev/null 2>&1
CERTIFICATE_FILE="${CERTIFICATE_PREFIX}0"
if [ ! -f "$CERTIFICATE_FILE" ]; then
  echo "could not extract signing certificate from $APP_BUNDLE" >&2
  exit 1
fi

ACTUAL=$(openssl x509 -inform DER -in "$CERTIFICATE_FILE" -noout -fingerprint -sha256 | sed 's/^[^=]*=//' | tr -d ':' | tr '[:lower:]' '[:upper:]')
if [ "$ACTUAL" != "$EXPECTED" ]; then
  echo "signing certificate fingerprint mismatch" >&2
  echo "expected: $EXPECTED" >&2
  echo "actual:   $ACTUAL" >&2
  exit 1
fi

echo "Verified $APP_BUNDLE with certificate SHA-256 $ACTUAL"
