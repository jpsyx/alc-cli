# AGENTS.md

This file is the single entry point for agents working in this repository.
Read it before changing code.

## What this project is

`alc` is a small Rust CLI for finding AA meetings published by New York
Inter-Group and reading Daily Reflections published by AA World Services. It
should make ordinary directory filters easy, make meetings available now or
soon especially easy to find, favor online meetings for a computer-based
workflow, and give in-person results useful Google Maps links.

This is an independent client. New York Inter-Group's directory and AA.org
remain the respective sources of truth. Never present a network, parsing, or
freshness failure as an authoritative empty result.

The command tree has two top-level workflows: `alc meeting` and
`alc daily-reflection`, whose visible alias is `alc daily`. A bare `alc` prints
help. A bare `alc meeting` behaves like `alc meeting now`.

## The docs contract

Whenever behavior, the command surface, a data-source assumption, or the module
shape changes, update the relevant document in the same change.

| If you change...                                 | Update...              |
| ------------------------------------------------ | ---------------------- |
| User-visible commands, flags, filters, or output | `docs/features.md`     |
| Module boundaries, data flow, or dependencies    | `docs/architecture.md` |
| Testing scope or strategy                        | `docs/testing.md`      |
| A non-obvious product or engineering choice      | `docs/decisions.md`    |

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
- Discover the current TSML JSON feed from the meeting page's
  `#tsml-ui[data-src]` element. Do not hardcode the generated cache filename.
- Cache the raw meeting directory at most once per `America/New_York` calendar
  day in the platform-standard user cache directory. Validate cached JSON
  through the normal parser before use, and keep cache writes best effort.
- Exclude inactive directory records. Include all active access modes by
  default, rank online-capable results first, and treat an online filter as
  including hybrid meetings.
- `meeting now` means meetings started within the last 30 minutes or starting
  within the next hour, inclusive and evaluated in `America/New_York`. Do not
  apply the 30-minute elapsed cutoff to any other meeting command.
- Date and time schedule shortcuts compose in either order. A time alone means
  today; `this week` aliases the next-seven-days `week` view; `evening` aliases
  `night`.
- Preserve and display the authoritative meeting page URL. Meeting details can
  change, so freshness and source provenance are part of correct output.
- Build every meeting display through the centralized `MeetingList` and
  `MeetingTable` renderer. Meeting-list commands must reuse its built-in pager,
  whole-record field matching, access shortcuts, relative timing, and
  original-view reset, and direct-output fallback.
- Display access as a bold semantic badge beside the meeting name, never as a
  separate table row. Keep the badge in whole-record matching.
- In pager filter editing, Ctrl+U clears the full query without exiting, while
  Backspace on an empty query returns to navigation.
- Show elapsed time beside every `IN PROGRESS` status as a non-bold red
  `[Started ... ago]` suffix.
- Daily Reflections come from AA.org's `/api/reflections/MM/DD` JSON route,
  whose `data` field contains HTML. Validate dates locally and validate the
  response's `data-date`; the endpoint normalizes invalid dates.
- Always display the Daily Reflection copyright attribution returned by AA.org
  and link to the official dated page. Use synthetic prose in test fixtures.
- Never infer a user's location. Distance features require an explicit origin;
  directions links may be generated directly from a listed address.
- Bump the pre-1.0 crate minor version for additive user-visible features and
  the patch version for compatible fixes or internal changes. Move
  `Cargo.lock` with `Cargo.toml`.

## Every command ships a `--help`

**Any command that takes arguments or does real work MUST
support `--help`, always.** The help must include:

1. **A one-line summary** of what the command does.
2. **The usage line / command synopsis** with proper conventions:
   `[optional]`, `<required>`, `...` for repeats, `|` for alternatives. Include
   the subcommand list when the command has subcommands.
3. **A parameter explanation** for every argument, subcommand, and flag: what it
   means and, where relevant, its default.
4. **At least a couple of worked examples**: real invocations showing how to
   accomplish different things with the command.

Scale the depth to the command. A trivial one-liner (say, a two-line git
wrapper) may ship a **concise** `--help`: the one-line summary, the usage line,
and a single example are enough. A **substantial** command (subcommands, several
flags, real behavior to explain) gets the full treatment above, every point. The
four requirements still all apply; "concise" shrinks each to its minimum, it
does not drop any. When in doubt, err toward the fuller help.

Handle `--help` (and `-h`) as the very first thing the command does, before any
side effect or tool check, and exit `0` after printing. Print the help to
stdout. A command invoked with missing required arguments must print the same
usage text to stderr and return non-zero, so the help is the single source of
truth for how to call it. That stderr-on-error rule is load-bearing for a
command whose stdout is a machine-consumable value (for example a bin that
prints a path for `cd "$(...)"`): sending usage to stdout instead would inject
the help text into the caller's command substitution.

**When you change a command, update its `--help` in the same edit, always,
without being asked.** If you add, remove, or rename a parameter or subcommand,
change a default, or change what the command is for, the `--help` text (summary,
synopsis, parameter list, and examples) must change to match in the same commit.
A `--help` that describes an older behavior is a bug. Treat keeping it accurate
as part of editing the command, exactly like keeping it green.

## CLI rules

1. **Before committing any CLI change, both must pass:**
   `cargo clippy --all-targets` (pedantic + nursery, zero warnings) and
   `cargo test`.

2. **Terminal output always prioritizes aesthetics and user-friendliness.**
   This is a top-priority, non-negotiable principle: every terminal-
   facing surface should be as pleasant and clear as we can make it.
   Two audiences, both first-class:

- **LLM-friendly:** there is a **CLI flag / subcommand for every possible
  action**, so an agent can drive `alc` fully non-interactively (no action is
  reachable _only_ through an interactive prompt).
- **Human-friendly:** when a human omits a required value, `alc` **drops into an
  interactive mode** (a themed prompt / guided walkthrough) instead of erroring
  When you add or change any command, provide both paths (flags for everything +
  an interactive fallback for missing values) and make the output beautiful.

3. **Aesthetics matter — theme every bit of CLI output.** All CLI
   output should look considered, not utilitarian. All color
   goes through the **`src/theme.rs` `Theme` semantic tokens** — `heading`,
   `accent`, `value`, `muted`, `success`, `warning`, `error`, `info`, `prompt` —
   chosen for _meaning_, never a raw ANSI escape inline. When you add or change
   CLI output, style it with the token that matches its role (a success message
   is `theme.success`, a command name is `theme.accent`, a hint is `theme.muted`,
   an interactive prompt label is `theme.prompt`, …), and be tasteful: color
   guides the eye, it doesn't paint everything. Get the theme via
   `Theme::active()` (color auto-gated off when stderr isn't a TTY or `NO_COLOR`
   is set); pass `Theme::dark(false)` in tests that assert on plain text.

   Meeting-search match highlighting uses the semantic `matched` token.
   Meeting timing uses the semantic `in_progress`, `time_started`, and
   `time_soon` tokens.

- **Design for dark terminals.** Terminals don't reliably expose a light/dark
  token, so we assume **dark** and use bright, high-contrast codes (never a
  dark foreground like plain blue `34` that vanishes on a dark background). A
  guard test enforces this. Adding a light (or other) theme later is just a
  new `Theme::…()` code table plus the selection in `Theme::active` — so keep
  all color decisions _in_ `Theme`, never scattered as literals.
