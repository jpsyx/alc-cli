# Architecture

## Current shape

The crate is organized by user workflow:

- `src/lib.rs` defines the nested Clap command tree and converts arguments into
  typed meeting or reflection requests;
- `src/schedule.rs` owns composable date and time shortcut arguments,
  normalization, validation, typed schedule options, and shared shortcut help;
- `src/meeting/` discovers NYIG's current structured feed, caches its raw
  directory response, validates and normalizes records, filters and ranks
  meetings, calculates current availability, builds Google Maps URLs, and
  presents centralized meeting tables through direct output or a pager;
- `src/daily_reflection.rs` validates dates, retrieves AA.org's structured
  response, verifies and parses its HTML fragment, and renders plain text;
- `src/text_wrap.rs` soft-wraps prose to a readable measure;
- `src/theme.rs` owns the dark-terminal semantic palette and the corresponding
  Clap help and error styles;
- `src/main.rs` is a thin process shell for clocks, environment overrides,
  progress, stdout, stderr, and exit codes;
- integration tests verify the compiled binary, typed requests, parsers, pure
  queries, rendering, and real local HTTP boundaries.

## Dependencies

The runtime remains synchronous:

- Clap defines and validates the command surface;
- Chrono supplies validated dates and times;
- `chrono-tz` evaluates current meeting availability in
  `America/New_York`, including daylight-saving transitions;
- Ureq provides blocking HTTPS with Rustls and a bounded global timeout;
- Serde and `serde_json` decode both upstream JSON formats;
- Scraper parses the NYIG directory page and AA.org reflection fragments;
- `html-escape` decodes character references embedded in NYIG JSON strings;
- URL resolves NYIG's advertised relative feed URL and safely constructs
  Google Maps query parameters;
- Crossterm supplies portable raw-mode input, alternate-screen management, key
  events, and terminal sizing for the meeting pager;
- `unicode-width` keeps table columns aligned by terminal display width rather
  than Unicode scalar count;
- Thiserror defines actionable source, response, schema, and markup errors.

No async runtime, cache-specific dependency, geocoder, prompt library, or
full terminal UI framework is used.

## Command and terminal flow

Clap command metadata defines summaries, usage, parameters, defaults, aliases,
and examples at every command level. Parsing obtains help, version, and usage
errors from the themed command builder. The current-date callback is lazy, so
help and meeting requests do not read the local date clock during parsing.
Schedule shorthand commands become a typed `MeetingSchedule`; the process
shell resolves its relative day against the single captured New York time.
Before Clap parsing, adjacent `this week` tokens normalize to the canonical
`week` shortcut. The first shorthand is a command and later shorthands are typed
positional filters, allowing date and time to appear in either order.

`Theme::active()` combines a fixed dark-terminal semantic palette with two
color gates: stderr must be a terminal and `NO_COLOR` must be absent. The same
theme styles application rendering, progress, failures, and Clap output. Tests
inject `Theme::dark(false)` for stable plain text or `Theme::dark(true)` for
exact style assertions.

All meeting-list commands construct the same `MeetingList` of `MeetingTable`
values. That model owns field selection, ordering, wrapping, styling, and
whole-record matching. It renders access as an inline semantic range in the
Meeting value, letting the renderer apply the correct bold access-badge color
without creating a separate row. It also records each table's access capability
so the pager can apply its access filter without re-querying or partially
rendering a record. Direct output renders it at a stable width. Interactive
output gives it to the pager, which renders at the current terminal width and
uses the same table code for every frame.

The pager separates its pure `PagerState` navigation, text query, and access
filter transitions from Crossterm IO. Filter editing maps Ctrl+U to a complete
query clear and maps Backspace on an empty query back to navigation. Access
shortcuts use the same capability semantics as command flags, so online and
in-person selections include hybrids. Reset replaces the transient pager state
with its default value while retaining the immutable `MeetingList` produced by
the original command.
It opens only when enabled and both stdin and stdout are terminals. A small
session guard restores raw mode, cursor visibility, and the previous screen on
normal exit or an error. Nonterminal output and `--no-pager` bypass terminal
control entirely.

No command currently has missing required data. Existing no-argument states
already mean help, meetings available now, an unfiltered meeting search, or
today's reflection. The process therefore has no prompt boundary yet and adds
no prompt dependency. If a later workflow requires input, terminal detection
and prompt IO belong in the process shell while the resulting choice remains a
typed library request.

## Meeting flow

The NYIG meeting page contains a `#tsml-ui` element whose `data-src` attribute
points at a generated TSML JSON cache. `alc` fetches the page first, resolves
that advertised URL, and then retrieves the complete directory. The generated
upstream filename is never hardcoded.

The source boundary stores the raw response in a versioned local cache keyed by
the compatible meeting page URL and the current `America/New_York` date. A
cache hit is parsed and validated before use. A missing, stale, malformed, or
incompatible cache is replaced after a successful fetch. Writes use a sibling
temporary file and rename where supported; cache write failures are best effort
and do not turn fetched meetings into a command failure.

The default file is `~/Library/Caches/alc/meetings.cache` on macOS,
`$XDG_CACHE_HOME/alc/meetings.cache` or `~/.cache/alc/meetings.cache` on Linux,
and `%LOCALAPPDATA%\alc\meetings.cache` on Windows. `ALC_CACHE_DIR` overrides
the platform cache root for isolated integration tests.

The parser validates required identity, weekday, and time fields before
creating normalized meetings. It keeps online and physical capabilities
separate so hybrid meetings retain both. Inactive entries are excluded by
queries. Text matching, filter composition, the 30-minute recent-start and
one-hour upcoming windows, relative schedule selection, online-first ranking,
table rendering, display-width wrapping, whole-meeting pager filtering, and
match range discovery are pure operations over normalized records.

The process shell captures the current New York time once per meeting command.
The query module determines whether each meeting is in progress or begins
within a supplied horizon. An in-progress availability retains the elapsed
minutes since the scheduled start. Rendering uses a 135-minute inclusive
horizon for relative schedule labels, while `meeting now` retains its separate
60-minute upcoming horizon and excludes elapsed values above 30 minutes.
Schedule shortcuts use a separate query path, so that exclusion cannot leak
into `today`, weekday, week, or time-of-day results. Semantic inline ranges let
schedule text remain a normal value. Only the upcoming suffix receives bold
yellow emphasis, and only the elapsed suffix beside `IN PROGRESS` receives
non-bold red emphasis.
The same range pipeline styles each access badge and gives a live text match
precedence over semantic timing and badge colors.

`ALC_NYIG_URL` can point the process shell at a compatible meeting page. The
integration suite combines this seam with an isolated cache root and a local
server; ordinary users use the official NYIG page.

## Daily Reflection flow

AA.org's own Daily Reflections page calls an internal
`/api/reflections/MM/DD` route. The route returns JSON, but its `data` property
is an HTML article. `alc` validates the requested calendar date locally,
requests the month and day, checks the article's `data-date`, extracts text,
and preserves copyright attribution.

Rendering wraps the title, body paragraphs, and copyright through
`text_wrap::wrap_prose`, a pure function with no terminal dependency. Each
wrapped line is styled individually so no escape sequence spans a line break.
The source URL keeps its own unwrapped line.

Both upstream sites are mutable external input. Retrieval and parsing errors
remain distinguishable from an authoritative empty result.
