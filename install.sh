#!/usr/bin/env bash
# Build and install one fixed nyintergroup binary. Re-running this script
# replaces the installed binary in place.
set -euo pipefail

usage() {
  cat <<'EOF'
install.sh: build nyintergroup and install it onto PATH.

Usage:
  ./install.sh [--help]

Options:
  -h, --help   Print this help and exit.

Environment:
  BIN_DIR      Installation directory. Default: $HOME/.local/bin.
EOF
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
  echo "error: 'cargo' not found; nyintergroup needs a Rust toolchain." >&2
  echo "       Install one from https://rustup.rs, then run this script again." >&2
  exit 1
fi

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_DIR="${BIN_DIR:-$HOME/.local/bin}"
mkdir -p "$INSTALL_DIR"
INSTALL_DIR="$(cd -- "$INSTALL_DIR" && pwd)"

echo "Building nyintergroup (release)..." >&2
(cd "$SCRIPT_DIR" && cargo build --release) >&2

INSTALLED_BINARY="$INSTALL_DIR/nyintergroup"
install -m 0755 "$SCRIPT_DIR/target/release/nyintergroup" "$INSTALLED_BINARY"
echo "installed nyintergroup -> $INSTALLED_BINARY"

case ":${PATH}:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo >&2
    echo "note: $INSTALL_DIR is not on your PATH, so nyintergroup will not be found yet." >&2
    echo "      Add it to your shell startup file, for example:" >&2
    echo "        export PATH=\"$INSTALL_DIR:\$PATH\"" >&2
    ;;
esac

