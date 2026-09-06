#!/usr/bin/env bash
# Build the release binary when needed, then execute it from any working
# directory while forwarding all arguments.
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
BIN="$SCRIPT_DIR/target/release/nyintergroup"
MANIFEST="$SCRIPT_DIR/Cargo.toml"
SRC_DIR="$SCRIPT_DIR/src"

needs_build=0
if [[ ! -x "$BIN" || "$MANIFEST" -nt "$BIN" ]]; then
  needs_build=1
elif [[ -n "$(find "$SRC_DIR" -name '*.rs' -newer "$BIN" -print -quit 2>/dev/null)" ]]; then
  needs_build=1
fi

if (( needs_build )); then
  echo "Building nyintergroup CLI..." >&2
  (cd "$SCRIPT_DIR" && cargo build --release) >&2
fi

exec "$BIN" "$@"

