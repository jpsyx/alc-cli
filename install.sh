#!/usr/bin/env bash
# Install or update the command from this checkout.
set -euo pipefail

usage() {
  printf '%s\n' \
    'Install alc for use from any directory.' \
    '' \
    'Usage: ./install.sh [--name <command>] [-h|--help]' \
    '' \
    'Options:' \
    '  --name <command>  Command filename (default: alc).' \
    '  -h, --help        Show this help without installing anything.' \
    '' \
    'Environment:' \
    '  BIN_DIR          Installation directory (default: $HOME/.local/bin).' \
    '' \
    'Examples:' \
    '  ./install.sh' \
    '  BIN_DIR="$HOME/bin" ./install.sh' \
    '  ./install.sh --name alc-dev'
}

for arg in "$@"; do
  case "$arg" in -h|--help) usage; exit 0 ;; esac
done

command_name="alc"
while (($#)); do
  case "$1" in
    --name)
      if (($# < 2)); then usage >&2; exit 2; fi
      command_name="$2"
      shift 2
      ;;
    *) usage >&2; exit 2 ;;
  esac
done
case "$command_name" in
  ''|[.-]*|*..*|*[!a-zA-Z0-9_.-]*) usage >&2; exit 2 ;;
esac
if ((${#command_name} > 100)); then usage >&2; exit 2; fi

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
if ! command -v cargo >/dev/null 2>&1; then
  echo "✗ 'cargo' was not found; alc needs a Rust toolchain." >&2
  echo "  Install one from https://rustup.rs, then run this script again." >&2
  exit 1
fi
bin_dir="${BIN_DIR:-$HOME/.local/bin}"
mkdir -p -- "$bin_dir"
bin_dir="$(cd -- "$bin_dir" && pwd)"
destination="$bin_dir/$command_name"
if [[ -d "$destination" ]]; then
  printf 'Cannot replace directory: %s\n' "$destination" >&2
  exit 1
fi
temporary="$(mktemp "$bin_dir/.install.XXXXXXXX")"
trap 'rm -f -- "$temporary"' EXIT
echo "● Building alc (release)..." >&2
(cd "$script_dir" && cargo build --release) >&2
install -m 0755 "$script_dir/target/release/alc" "$temporary"
mv -f -- "$temporary" "$destination"
[[ -f "$destination" && -x "$destination" ]]
printf '✓ Installed %s\n  %s\n' "$command_name" "$destination"
case ":${PATH}:" in
  *":$bin_dir:"*) ;;
  *)
    echo >&2
    echo "! The installation directory is not on PATH." >&2
    echo "  Add this to your shell startup file:" >&2
    echo "  export PATH=\"$bin_dir:\$PATH\"" >&2
    ;;
esac
