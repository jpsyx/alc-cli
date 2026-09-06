# Testing

Development follows strict red/green TDD. Every production behavior begins with
the smallest failing test, followed by the smallest implementation and a green
refactor pass.

Use focused unit tests for pure parsing, filtering, ranking, time-window, and URL
logic. Use fixtures for representative upstream HTML once a data-access design
is chosen. Keep live network tests out of the default fast suite because the
directory is external and mutable. Use a small number of integration tests for
the compiled CLI contract.

The full local verification gate is:

```sh
cargo test --release
cargo clippy --release --all-targets -- -D warnings
cargo fmt --all -- --check
```

