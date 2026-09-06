# ALC

`alc` is an independent Rust CLI for finding AA meetings listed by
[New York Inter-Group](https://www.nyintergroup.org/meetings/) and reading
[AA Daily Reflections](https://www.aa.org/daily-reflections).

Meeting information changes. The upstream directory and each meeting's source
page remain authoritative. This project is not affiliated with Alcoholics
Anonymous or New York Inter-Group.

## Meetings

Find meetings started within the last 30 minutes or starting within the next
hour:

```sh
alc meeting
alc meeting now
alc meeting now --type online --limit 10
```

`alc meeting` is an alias for `alc meeting now`. Times are evaluated in New
York. Results include online, hybrid, and in-person meetings by default, with
online-capable meetings first. `--type online` includes hybrid meetings. The
30-minute elapsed cutoff is inclusive and applies only to `meeting now` and the
bare `meeting` alias.

Use composable schedule shortcuts for broader date and time views:

```sh
alc meeting today
alc meeting tomorrow night
alc meeting week morning
alc meeting this week morning
alc meeting tuesday
alc meeting evening this week
alc meeting afternoon today --type online
```

Date and time shortcuts work in either order. `this week` aliases `week`, and
`evening` aliases `night`. A time without a date means today. `week` covers the
next seven days beginning today. Morning uses NYIG's 4:00 AM to noon range,
afternoon uses its overlapping 11:00 AM to 5:00 PM range, and night/evening
combines its evening and night ranges from 4:00 PM through 5:00 AM. Schedule
shortcuts accept `--type`, `--region`, `--limit`, `--from`, and `--no-pager`.
Unlike `meeting now`, these views do not exclude meetings that started more
than 30 minutes ago.

Search the complete active directory:

```sh
alc meeting find
alc meeting find "turning point"
alc meeting find --weekday sunday --time morning --type online
alc meeting find --region brooklyn --type in-person --limit 10
alc meeting find "midtown" --from "10001"
```

`find` supports a text query plus `--weekday`, `--time`, `--type`, `--region`,
`--limit`, and `--from`. Time filters match NYIG's overlapping `morning`,
`midday`, `evening`, and `night` categories. In-person and hybrid results
include a Google Maps directions link. `--from` adds an explicit origin to
that link; `alc` never infers a user's location.

Every meeting is rendered as a complete two-column table. Its access mode is a
bold, color-coded badge next to the meeting name, such as `[Online]`,
`[Hybrid]`, or `[In person]`. When both input and output are interactive
terminals, meeting lists open in a built-in pager:

| Key | Action |
| --- | --- |
| `j`, `k`, down arrow, up arrow | Move one line. |
| `d`, `u` | Move half a page down or up. |
| `/` | Edit a live filter that searches every displayed field. |
| `Ctrl+U` | Clear the filter text while continuing to edit. |
| `Backspace` | Delete one character, or exit editing when the filter is empty. |
| `Enter`, `Esc` | Finish editing the filter and return to navigation. |
| `G` | Jump to the end of the filtered list. |
| `a` | Show all access modes. |
| `h` | Show hybrid meetings. |
| `p` | Show in-person-capable meetings, including hybrid. |
| `o` | Show online-capable meetings, including hybrid. |
| `r` | Reset filters and scroll position to the original command view. |
| `q` | Quit while navigating. While editing, `q` is filter text. |

A filter keeps the complete table for every matching meeting and highlights
each matching fragment. Text and access filters combine, and `a` clears only
the access filter. While editing a text filter, `a`, `h`, `p`, and `o` remain
ordinary search text. Use `--no-pager` to print tables directly. Redirected or
piped output automatically uses the same direct form.

`r` clears the pager's text and access filters, exits filter editing, and
returns to the top. It preserves the original command query, flags, result
limit, and ordering because those define the list that the pager received.

Schedule text uses the normal value emphasis rather than dimmed secondary text.
Meetings already underway show `IN PROGRESS` in bold green followed by a
non-bold red elapsed time such as `[Started 8 minutes ago]`. Meetings starting
within 2 hours 15 minutes, inclusive, append a bold yellow relative time such as
`[In 1 hour 27 minutes]`. Relative timing uses New York time.

The full meeting directory is fetched at most once per New York calendar day.
Later meeting commands reuse the validated local cache:

| Platform | Cache file |
| --- | --- |
| macOS | `~/Library/Caches/alc/meetings.cache` |
| Linux | `$XDG_CACHE_HOME/alc/meetings.cache`, or `~/.cache/alc/meetings.cache` |
| Windows | `%LOCALAPPDATA%\alc\meetings.cache` |

Deleting the file forces the next meeting command to refresh it. Daily
Reflections are not stored in this cache.

## Daily Reflections

```sh
alc daily
alc daily-reflection
alc daily --date 2024-02-29
alc daily --date 09-06
```

Dates accept `YYYY-MM-DD` or `MM-DD`. When the year is omitted, `alc` uses the
current local year. With no date, it uses today's local date. Output includes
AA World Services' attribution and a link to the official source page.

A bare `alc` prints help.

Every command has `-h` and `--help` output with its options, defaults, pager
controls where relevant, and worked examples. Interactive terminals receive a
restrained, high-contrast dark-terminal theme. Piped output stays plain, and
setting `NO_COLOR` disables color explicitly.

## Run from source

Rust 1.85 or newer is required.

```sh
./run.sh --help
./run.sh meeting find --weekday sunday --time morning --type online
./run.sh daily
```

`run.sh --help` and `install.sh --help` work before tool or build checks.

## Install

```sh
./install.sh
alc --help
```

The default destination is `~/.local/bin/alc`. Set `BIN_DIR` to choose
another directory.

## Develop

```sh
npx skills experimental_install
cargo test --release
cargo clippy --release --all-targets -- -D warnings
cargo fmt --all -- --check
```

Read `AGENTS.md` and `docs/README.md` before changing behavior.
