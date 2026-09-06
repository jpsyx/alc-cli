# NY Intergroup CLI

`nyintergroup` is an independent Rust CLI for finding AA meetings listed by
[New York Inter-Group](https://www.nyintergroup.org/meetings/).

The repository is at its initial foundation stage. The binary currently exposes
help and version information while the meeting-query interface is designed.
The intended product is deliberately focused:

- query meetings with basic day, time, and format filters;
- find meetings happening now or starting soon;
- make online meetings especially quick to identify and join;
- provide Google Maps directions for in-person meetings, with distance support
  only when the user supplies an origin.

Meeting information changes. The upstream directory and each meeting's source
page remain authoritative. This project is not affiliated with Alcoholics
Anonymous or New York Inter-Group.

## Run from source

Rust 1.85 or newer is required.

```sh
./run.sh --help
```

## Install

```sh
./install.sh
nyintergroup --help
```

The default destination is `~/.local/bin/nyintergroup`. Set `BIN_DIR` to choose
another directory.

## Develop

```sh
npx skills experimental_install
cargo test --release
cargo clippy --release --all-targets -- -D warnings
cargo fmt --all -- --check
```

Read `AGENTS.md` and `docs/README.md` before changing behavior.

