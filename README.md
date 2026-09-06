# ALC

`alc` is an independent Rust CLI for finding AA meetings listed by
[New York Inter-Group](https://www.nyintergroup.org/meetings/) and reading
[AA Daily Reflections](https://www.aa.org/daily-reflections).

Meeting information changes. The upstream directory and each meeting's source
page remain authoritative. This project is not affiliated with Alcoholics
Anonymous or New York Inter-Group.

## Meetings

Find meetings in progress or starting within the next hour:

```sh
alc meeting
alc meeting now
alc meeting now --type online --limit 10
```

`alc meeting` is an alias for `alc meeting now`. Times are evaluated in New
York. Results include online, hybrid, and in-person meetings by default, with
online-capable meetings first. `--type online` includes hybrid meetings.

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

Every meeting is rendered as a complete two-column table. When both input and
output are interactive terminals, meeting lists open in a built-in pager:

| Key | Action |
| --- | --- |
| `j`, `k`, down arrow, up arrow | Move one line. |
| `d`, `u` | Move half a page down or up. |
| `/` | Edit a live filter that searches every displayed field. |
| `Enter`, `Esc` | Finish editing the filter and return to navigation. |
| `G` | Jump to the end of the filtered list. |
| `q` | Quit while navigating. While editing, `q` is filter text. |

A filter keeps the complete table for every matching meeting and highlights
each matching fragment. Use `--no-pager` to print tables directly. Redirected
or piped output automatically uses the same direct form.

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
