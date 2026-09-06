# Testing

Development follows strict red/green TDD. Every production behavior begins
with the smallest failing test, followed by the smallest implementation and a
green refactor pass.

Pure integration tests cover typed command parsing, meeting normalization,
case-insensitive filters, hybrid access semantics, online-first ranking,
current weekly time calculations, HTML-entity normalization, direction URL
generation, complete table rendering, Unicode display-width alignment, and
semantic terminal styles.

Compiled-CLI tests exercise help at every command level, including examples and
implicit defaults. A lazy clock test proves help resolution does not request a
date. Theme tests cover all semantic roles, dark-terminal contrast, plain output,
and the independent terminal and `NO_COLOR` gates.

Pager unit tests cover line and half-page movement, viewport clamping, jump to
start and end, quit behavior, live search editing, and less-like key mapping.
Pure frame tests prove that a filter searches every field, retains the complete
matching meeting table, removes nonmatching meetings, and highlights every
case-insensitive match. TTY gating tests prove that direct output remains the
fallback when paging is disabled or either stream is not interactive.

Local TCP servers exercise the real synchronous HTTP client. Meeting tests
serve both a synthetic HTML page and its advertised synthetic JSON feed. They
verify same-day cache reuse after the server stops, next-day refresh, malformed
cache replacement, best-effort writes, and end-to-end CLI reuse. Daily
Reflection tests serve synthetic HTML inside a representative JSON envelope.
The default suite requires neither upstream site and stores no copyrighted
reflection prose.

Shell entry-point tests invoke `run.sh --help` and `install.sh --help` with an
empty `PATH`, proving help completes before tool and build checks. The installer
also runs against an isolated temporary directory to verify its build, success,
and warning output without modifying the user's installation.

Live smoke tests may be run manually against both official sources after the
default suite passes. The full local verification gate is:

```sh
cargo test --release
cargo clippy --release --all-targets -- -D warnings
cargo fmt --all -- --check
```
