#!/usr/bin/env bash
# Build and install one fixed alc binary. Re-running this script
# replaces the installed binary in place.
set -euo pipefail

usage() {
  printf '%s\n' \
    'Install alc for use from any directory.' \
    '' \
    'Usage:' \
    '  ./install.sh [--help]' \
    '' \
    'Options:' \
    '  -h, --help   Print help and exit.' \
    '' \
    'Environment:' \
    '  BIN_DIR      Installation directory. Default: $HOME/.local/bin.' \
    '' \
    'Examples:' \
    '  ./install.sh' \
    '  BIN_DIR="$HOME/bin" ./install.sh'
}

case "${1:-}" in
  -h | --help)
    usage
    exit 0
    ;;
  "") ;;
  *)
    usage >&2
    exit 1
    ;;
esac

if ! command -v cargo >/dev/null 2>&1; then
  echo "✗ 'cargo' was not found; alc needs a Rust toolchain." >&2
  echo "  Install one from https://rustup.rs, then run this script again." >&2
  exit 1
fi

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_DIR="${BIN_DIR:-$HOME/.local/bin}"
mkdir -p "$INSTALL_DIR"
INSTALL_DIR="$(cd -- "$INSTALL_DIR" && pwd)"

echo "● Building alc (release)..." >&2
(cd "$SCRIPT_DIR" && cargo build --release) >&2

INSTALLED_BINARY="$INSTALL_DIR/alc"
install -m 0755 "$SCRIPT_DIR/target/release/alc" "$INSTALLED_BINARY"
printf '✓ Installed alc\n  %s\n' "$INSTALLED_BINARY"

case ":${PATH}:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo >&2
    echo "! The installation directory is not on PATH." >&2
    echo "  Add this to your shell startup file:" >&2
    echo "  export PATH=\"$INSTALL_DIR:\$PATH\"" >&2
    ;;
esac
