# AGENTS.md

This file is the single entry point for agents working in this repository.
Read it before changing code.

## What this project is

`nyintergroup` is a small Rust CLI for finding AA meetings published by New
York Inter-Group. It should make ordinary directory filters easy, make meetings
available now or soon especially easy to find, favor online meetings for a
computer-based workflow, and give in-person results useful Google Maps links.

This is an independent client. New York Inter-Group's directory remains the
source of truth. Never present a network, parsing, or freshness failure as an
authoritative claim that no meetings exist.

The initial repository is intentionally only a CLI foundation. Do not choose a
scraping strategy, caching policy, location provider, or broad command surface
until the behavior is agreed and recorded in `docs/`.

## The docs contract

Whenever behavior, the command surface, a data-source assumption, or the module
shape changes, update the relevant document in the same change.

| If you change... | Update... |
| --- | --- |
| User-visible commands, flags, filters, or output | `docs/features.md` |
| Module boundaries, data flow, or dependencies | `docs/architecture.md` |
| Testing scope or strategy | `docs/testing.md` |
| A non-obvious product or engineering choice | `docs/decisions.md` |

Documentation is the source of truth for what the CLI does and why. Code is the
source of truth for how.

## Red/green TDD: the iron law

No production behavior lands without a failing test written first.

1. RED: write the smallest test for the next behavior and run it to observe the
   expected failure.
2. GREEN: write the simplest implementation that makes that test pass.
3. REFACTOR: clean up with the suite green, then run it again.

For a bug, first add a failing regression test. Put decisions such as time-window
matching, filter composition, meeting ranking, URL generation, and parsing into
pure functions. Test those functions without networking, clocks, terminals, or
the filesystem wherever possible.

## Build, run, and test

```sh
./run.sh --help
cargo test --release
cargo clippy --release --all-targets -- -D warnings
cargo fmt --all -- --check
```

`run.sh` rebuilds the release binary when Rust sources or the manifest change,
then executes it. Users should not need to type `cargo run`.

`install.sh` is the idempotent installer and updater. It must work from any
clone, diagnose a missing Rust toolchain, install one fixed binary name, and
warn when its destination is not on `PATH`.

## Agent development skills

This repository pins contributor skills in `skills-lock.json`. Materialized
copies are local build artifacts and are gitignored. Restore the exact set with:

```sh
npx skills experimental_install
```

The set matches the Brain CLI repository:

- `rust-skills`
- the Actionbook Rust suite: `coding-guidelines`, `domain-cli`,
  `rust-call-graph`, `rust-code-navigator`, `rust-deps-visualizer`,
  `rust-learner`, `rust-refactor-helper`, `rust-router`,
  `rust-symbol-analyzer`, `rust-trait-explorer`, and `unsafe-checker`
- `test-driven-development` and `systematic-debugging`
- `repo-product-manager`

Invoke `rust-skills` when writing, reviewing, or refactoring Rust. Use
`test-driven-development` for product changes and `systematic-debugging` for
bugs or unexpected test failures. Do not rely on globally installed copies.

## House rules

- No `unsafe`; `Cargo.toml` enforces this.
- Keep Clippy clean with `pedantic` and `nursery` enabled.
- Keep dependencies few and deliberate. Explain each non-obvious dependency in
  `docs/architecture.md`.
- Keep impure shells thin. Network, clock, filesystem, location, and terminal
  boundaries should call pure decision logic.
- Prefer small, single-responsibility modules. A Rust file approaching 400
  production lines is a prompt to find a real seam and split it.
- Comments explain non-obvious reasons, not mechanics already clear from names.
- Every user action must be available non-interactively through commands and
  flags. Interactive conveniences may supplement, but never replace, that path.
- Default output should be concise and human-friendly. Machine-readable output
  must be explicit, stable, and free of progress or diagnostic text.
- Diagnostics and progress go to stderr. Meeting results, explicit structured
  output, help, and version output go to stdout.
- A potentially slow network operation narrates its major phases so the user
  never has to guess whether the CLI is stalled.
- Centralize any future color decisions in semantic theme tokens. Respect
  `NO_COLOR` and non-TTY output; never scatter ANSI literals through modules.
- Treat online, in-person, and hybrid meetings as distinct capabilities. Do not
  discard either side of a hybrid listing.
- Preserve and display the authoritative meeting page URL. Meeting details can
  change, so freshness and source provenance are part of correct output.
- Never infer a user's location. Distance features require an explicit origin;
  directions links may be generated directly from a listed address.
- Bump the pre-1.0 crate minor version for additive user-visible features and
  the patch version for compatible fixes or internal changes. Move
  `Cargo.lock` with `Cargo.toml`.

