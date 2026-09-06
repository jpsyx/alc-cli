# Features

## Command tree

```text
alc
├── meeting
│   ├── now
│   └── find [QUERY]
└── daily-reflection (alias: daily)
```

A bare `alc` prints help. `alc meeting` is equivalent to
`alc meeting now`.

Every command supports `-h` and `--help`. Help includes a summary, usage,
every argument and option, relevant defaults, and worked examples. Help is
resolved before clocks, network requests, filesystem writes, or build checks.
The redundant generated `help` subcommand is disabled, leaving one consistent
help surface.

## Meeting availability

`alc meeting now` lists active meetings in progress or beginning within the
next hour. Schedules use New York time. Online and hybrid meetings appear
before physical-only meetings, but all active access modes are included unless
`--type` restricts them.

Both `now` and `find` support `--type`, `--limit`, and `--from`. `--type`
accepts `online`, `in-person`, or `hybrid`. Online and in-person each include
hybrid meetings. `--from ADDRESS` adds an explicit origin to physical Google
Maps direction links.

## Meeting search

`alc meeting find` searches the complete active NYIG directory. Its optional
positional query matches meeting names, groups, locations, addresses, regions,
and slugs without regard to case.

Additional filters are:

- `--weekday sunday` through `--weekday saturday`;
- `--time morning`, `midday`, `evening`, or `night`, using NYIG's overlapping
  time ranges;
- `--type online`, `in-person`, or `hybrid`;
- `--region TEXT`;
- `--limit NUMBER`, defaulting to 20;
- `--from ADDRESS` for directions from a supplied origin.

Results show access mode, schedule, online join details, physical location,
directions, region, meeting type codes, and the authoritative NYIG source URL
when available. Each result is one complete two-column table built by the same
renderer for every meeting-list command.

## Meeting cache

The complete NYIG directory is fetched at most once per New York calendar day.
All `meeting now` and `meeting find` queries reuse that day's validated cache.
A missing, stale, malformed, or incompatible cache triggers a fresh download.
Failure to write the cache does not hide otherwise valid meeting results.

The cache file follows each platform's conventional user cache location:

| Platform | Cache file |
| --- | --- |
| macOS | `~/Library/Caches/alc/meetings.cache` |
| Linux | `$XDG_CACHE_HOME/alc/meetings.cache`, or `~/.cache/alc/meetings.cache` |
| Windows | `%LOCALAPPDATA%\alc\meetings.cache` |

Deleting the file forces a refresh on the next meeting command.

## Terminal experience

Human-readable results use a centralized semantic theme for headings, labels,
values, status, matches, warnings, and errors. The active palette uses bright
colors chosen for dark terminals. Color is enabled only when stderr is a
terminal and `NO_COLOR` is unset, so redirected and piped output remains plain.

Meeting lists use a built-in pager when stdin and stdout are terminals and the
request does not include `--no-pager`. The pager opens in the alternate screen
and restores the prior terminal screen and mode when it exits. Its controls are:

| Key | Action |
| --- | --- |
| `j`, `k`, down arrow, up arrow | Move one rendered line. |
| `d`, `u`, Page Down, Page Up | Move half a viewport. |
| `/` | Enter live filter editing. |
| `Enter`, `Esc` | Finish filter editing. |
| `G`, End | Jump to the end. |
| `g`, Home | Jump to the start. |
| `q` | Quit from navigation mode. |

The live filter performs a case-insensitive match across every label and value
in a meeting table. A match retains the entire meeting rather than selecting a
single row, and every matching text fragment is highlighted. While editing a
filter, printable keys such as `q` remain search text. `--no-pager`, redirected
stdin, or redirected stdout prints the same complete tables directly.

Slow retrieval begins with an informational status line on stderr. Results stay
on stdout, and failures use a clear error marker on stderr with a nonzero exit.

Every current command has a complete action when optional input is omitted:

| Invocation | Default action |
| --- | --- |
| `alc` | Print root help. |
| `alc meeting` | Find meetings available now or within one hour. |
| `alc meeting find` | Search all active meetings. |
| `alc daily` | Read today's reflection. |

Because no workflow currently lacks a required value, these invocations do not
prompt. Every action remains fully available through commands and flags. A
future action with missing required input must offer a themed terminal prompt
while retaining a complete non-interactive flag path.

## Daily Reflections

- `alc daily` and `alc daily-reflection` retrieve today's reflection;
- `--date YYYY-MM-DD` selects a complete date;
- `--date MM-DD` uses the current local year.

Invalid calendar dates fail before a network request. Output contains the
title, displayed date, body, official source URL, and AA World Services
copyright attribution.

## Failure behavior

Network, malformed-source, and schema failures print an error to stderr and
exit nonzero when no valid current-day cache can serve the request. A successful
meeting query with no matches prints an explicit empty-result message to stdout.
