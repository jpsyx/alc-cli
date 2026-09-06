# Decisions

## 2026-09-06: Start with a minimal Rust CLI foundation

The project begins as a Rust 2024 crate with a Rust 1.85 minimum toolchain,
Clap for command metadata, strict Clippy lints, and `unsafe` forbidden. This
matches the established Brain CLI baseline while avoiding dependencies that no
agreed behavior requires.

The installed command is `nyintergroup`. The package remains
`nyintergroup-cli` so its repository purpose is explicit without burdening the
user-facing command.

## 2026-09-06: Defer meeting-source implementation

The source directory clearly represents online, in-person, and hybrid meetings,
but setup alone does not justify choosing between HTML retrieval, a structured
endpoint, caching, or a browser-backed approach. Those choices will follow the
feature discussion and be documented with their failure and freshness model.

