# Decisions

## 2026-09-06: Start with a minimal Rust CLI foundation

The project uses Rust 2024 with a Rust 1.85 minimum toolchain, Clap for command
metadata, strict Clippy lints, and `unsafe` forbidden. This matches the Brain
CLI baseline.

## 2026-09-06: Name the CLI `alc`

The installed command and Rust package are named `alc`. The rejected `aa` name
would shadow macOS's `/usr/bin/aa` archive utility on common `PATH`
configurations.

## 2026-09-06: Use explicit top-level workflow commands

The two top-level commands are `meeting` and `daily-reflection`, with `daily`
as a visible alias for the latter. A bare `alc` prints help. A bare
`alc meeting` behaves like `alc meeting now`.

This supersedes the earlier decision that a bare `alc` would run a current
meeting query. It also supersedes the temporary `--daily-reflection` and `-dr`
flags.

## 2026-09-06: Discover NYIG's structured feed

NYIG's meeting page uses the Twelve Step Meeting List interface and advertises
a generated JSON cache through `#tsml-ui[data-src]`. `alc` resolves that URL on
each invocation instead of hardcoding a mutable cache filename or scraping a
rendered table.

The feed includes online, in-person, hybrid, and inactive records. Queries
exclude inactive entries, include every active access mode by default, and
rank online-capable meetings first. Online and in-person filters each include
hybrids because a hybrid meeting has both capabilities.

## 2026-09-06: Bound the `meeting now` availability window

`meeting now` returns meetings started no more than 30 minutes ago plus meetings
starting within the next 60 minutes. Both boundaries are inclusive. Recurring
schedules are evaluated in `America/New_York`. Online-capable results are
ranked before physical-only results. The elapsed cutoff is deliberately local
to `now`; broader schedule and `find` views retain every matching meeting.

## 2026-09-06: Compose schedule shortcuts in either order

Date shortcuts and time-of-day shortcuts form one typed `MeetingSchedule`.
The first shortcut is represented as a Clap subcommand and any later shortcuts
as validated positional values, then both are folded into the same day and time
fields. This makes `tomorrow night` and `evening tomorrow` equivalent without a
permutation-specific command tree. A narrow pre-parse normalization turns the
two adjacent tokens `this week` into the canonical `week`; `evening` maps to
`night`.

A time shortcut alone means today. `week` covers today through the next six
days. Morning and afternoon reuse NYIG's overlapping morning and midday ranges;
night combines its evening and night ranges. Distinct competing date or time
selectors fail instead of making ordering change which filter wins.

## 2026-09-06: Keep location explicit

Physical and hybrid results receive Google Maps directions links from their
listed address. `--from` adds a user-supplied origin. The CLI does not request,
infer, or store a user's location, and it does not call a geocoder.

## 2026-09-06: Use AA.org's internal Daily Reflections endpoint

AA.org's Daily Reflections page calls `/api/reflections/MM/DD`. The route
returns JSON with an HTML reflection fragment. Public Drupal JSON:API routes
are not enabled, so `alc` consumes the same internal route as the official
page while treating it as mutable external input.

The endpoint does not accept a year and normalizes invalid dates. `alc` accepts
`YYYY-MM-DD` or `MM-DD`, supplies the current local year when omitted,
validates the full date before fetching, and checks the returned month and day.

## 2026-09-06: Cache the full meeting directory by New York date

Meeting commands share one raw-directory cache and fetch NYIG at most once per
`America/New_York` calendar day. Platform-standard user cache directories keep
disposable data out of configuration and document storage. The cache records a
format version, date, and page URL, and its JSON must still pass the normal
directory parser before use.

Malformed, stale, and incompatible entries are cache misses. Cache writes are
best effort because a read-only cache directory should not prevent a successful
network response from reaching the user. The smaller Daily Reflection response
is outside this cache because the requested date can vary independently.

## 2026-09-06: Centralize a dark-terminal semantic theme

All terminal styling is owned by `Theme` roles rather than call-site ANSI
literals. The initial palette uses bright foregrounds suitable for dark
terminals. Color requires terminal stderr and an unset `NO_COLOR`; otherwise
the same renderers produce stable plain text for pipes and automation. Clap
help and usage errors share the palette.

Progress uses a compact informational marker, failures use an error marker, and
content uses color selectively for hierarchy and actions. This preserves
readability without turning every character into decoration.

## 2026-09-06: Prompt only when required input is genuinely missing

No existing workflow requires an interactive prompt. Bare `alc` intentionally
prints help, bare `alc meeting` means `meeting now`, an unfiltered `meeting
find` searches all active meetings, and an undated `daily` uses today. Prompting
for any of those would replace a useful documented command with an unnecessary
interaction.

Future workflows with required values must provide both a themed terminal
prompt when those values are omitted and explicit arguments or flags for
non-interactive use. This keeps human guidance and full automation compatible.

## 2026-09-06: Use one built-in meeting table and pager pipeline

Every meeting-list command builds the same complete two-column meeting tables.
Interactive terminals browse those tables in a small built-in pager, while
`--no-pager` and redirected streams print the same tables directly. This keeps
future meeting-output changes centralized and makes scripted output predictable.

The pager filters whole meeting records rather than individual rendered lines.
It searches every displayed label and value, retains the complete table for a
matching record, and highlights each matching fragment. Crossterm provides only
the portable terminal boundary; navigation, filtering, layout, and frame
rendering remain testable pure logic. An external `less` process was rejected
because it could not provide whole-record live filtering and highlighting with
the same behavior on every supported platform.

## 2026-09-06: Add access shortcuts and near-term schedule emphasis

The built-in pager uses `a`, `h`, `p`, and `o` as direct access filters while
in navigation mode. Online and in-person filters include hybrid meetings to
match the established command-filter capability semantics. These filters
combine with the live text query, and the same keys remain ordinary characters
while that query is being edited.

Meeting schedules remain visually primary. An underway meeting receives an
uppercase bold green status followed by a non-bold red elapsed-time suffix.
Elapsed minutes come from the same weekly schedule calculation that decides
whether the meeting is underway, so both `now` and `find` present consistent
timing. An upcoming meeting receives a bold yellow relative suffix only when
its start is at most 135 minutes away. The 135-minute horizon provides useful
near-term context in a general `find` list without changing the narrower
upcoming membership rule for `meeting now`.

The `r` shortcut resets only transient pager state. It restores all access
modes, clears the live text query, exits query editing, and returns to the top
of the list. It deliberately preserves command-line filters and limits by
retaining the immutable meeting list that originally entered the pager.

Access is displayed as a bold semantic badge directly beside the meeting name.
This removes a repetitive table row while keeping the capability visible at the
first scanning point. The badge remains part of the searchable Meeting value,
and match highlighting takes precedence when a query overlaps it.

Filter editing follows familiar terminal input conventions: Ctrl+U clears the
entire query without leaving editing, and Backspace on an already empty query
returns to navigation. This gives both a fast restart and a one-key exit without
overloading the printable navigation shortcuts used as search text.

## 2026-09-17: Wrap Daily Reflection prose to an approximate measure

Reflection paragraphs arrive as single long lines and previously filled the
terminal's full width, which reads poorly. `alc daily` now wraps prose near 80
columns.

The limit is deliberately soft. A hard 80-column break moves a word down to
save one or two columns and leaves a visibly short line behind, so the
algorithm keeps a word whose line ends within five columns of the target, and
keeps an even longer overrun when breaking would leave a line under 76 columns.
Words are never split: a long word or URL overhangs rather than becoming
unsearchable fragments. The reflection's source URL keeps its own line for the
same reason.

Wrapping lives in its own pure module rather than reusing the meeting table's
hard-width `wrap_text`, because a table column must respect an exact width
while prose should not.
