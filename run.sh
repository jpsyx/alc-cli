#!/usr/bin/env bash
# Build the release binary when needed, then execute it from any working
# directory while forwarding all arguments.
set -euo pipefail

usage() {
  printf '%s\n' \
    'Find AA meetings and read Daily Reflections from the terminal.' \
    '' \
    'Usage:' \
    '  ./run.sh <COMMAND>' \
    '' \
    'Commands:' \
    '  meeting           Find New York Inter-Group meetings. Aliases: mtg, m.' \
    '  daily-reflection  Read an AA Daily Reflection. Aliases: daily, d.' \
    '' \
    'Options:' \
    '  -h, --help        Print help and exit.' \
    '  -V, --version     Print the alc version.' \
    '' \
    'Examples:' \
    '  ./run.sh meeting' \
    '  ./run.sh meeting find --weekday sunday --type online' \
    '  ./run.sh daily' \
    '  ./run.sh m today'
}

case "${1:-}" in
  -h | --help)
    usage
    exit 0
    ;;
esac

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
BIN="$SCRIPT_DIR/target/release/alc"
MANIFEST="$SCRIPT_DIR/Cargo.toml"
SRC_DIR="$SCRIPT_DIR/src"

needs_build=0
if [[ ! -x "$BIN" || "$MANIFEST" -nt "$BIN" ]]; then
  needs_build=1
elif [[ -n "$(find "$SRC_DIR" -name '*.rs' -newer "$BIN" -print -quit 2>/dev/null)" ]]; then
  needs_build=1
fi

if (( needs_build )); then
  echo "● Building alc CLI..." >&2
  (cd "$SCRIPT_DIR" && cargo build --release) >&2
fi

exec "$BIN" "$@"
