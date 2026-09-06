# Features

## Command tree

```text
alc
├── meeting
│   ├── now
│   ├── find [QUERY]
│   ├── today | tomorrow | week
│   ├── sunday | monday | ... | saturday
│   └── morning | afternoon | night (alias: evening)
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

`alc meeting now` lists active meetings that started within the last 30 minutes
or begin within the next hour. Both boundaries are inclusive. The elapsed-time
cutoff applies only to `meeting now` and its bare `meeting` alias. Schedules use
New York time. Online and hybrid meetings appear before physical-only meetings,
but all active access modes are included unless `--type` restricts them.

Schedule values use normal primary text. An underway meeting adds a bold green,
uppercase `IN PROGRESS` status followed by a non-bold red elapsed time such as
`[Started 8 minutes ago]`. Any displayed meeting starting within 2 hours 15
minutes, inclusive, appends a bold yellow relative start time to its schedule.
Relative timing applies to both `now` and `find` results and is calculated in
New York time.

Both `now` and `find` support `--type`, `--limit`, and `--from`. `--type`
accepts `online`, `in-person`, or `hybrid`. Online and in-person each include
hybrid meetings. `--from ADDRESS` adds an explicit origin to physical Google
Maps direction links.

## Meeting schedule shortcuts

Date shortcuts are `today`, `tomorrow`, `week`, and every weekday name. `this
week` aliases `week`, which covers the next seven days beginning with the
current New York date. Time shortcuts are `morning`, `afternoon`, and `night`,
with `evening` as an alias for `night`. A time shortcut without a date defaults
to today.

Date and time shortcuts compose in either order, so `meeting tomorrow night`,
`meeting evening tomorrow`, `meeting this week morning`, and `meeting morning
this week` select identical pairs of filters. Morning uses NYIG's 4:00 AM to
noon range. Afternoon uses its overlapping 11:00 AM to 5:00 PM range.
Night/evening combines the NYIG evening and night ranges, covering 4:00 PM
through 5:00 AM. Selecting more than one distinct date or time shortcut is an
argument error.

Schedule shortcuts accept `--type`, `--region`, `--limit`, `--from`, and
`--no-pager`. They show every matching scheduled meeting, including meetings
that began over 30 minutes ago; the recent-start cutoff belongs only to `now`.

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

Results show access mode as a bold, color-coded badge beside the meeting name,
followed by schedule, online join details, physical location, directions,
region, meeting type codes, and the authoritative NYIG source URL when
available. Access does not occupy a separate table row. Each result is one
complete two-column table built by the same renderer for every meeting-list
command.

## Meeting cache

The complete NYIG directory is fetched at most once per New York calendar day.
All meeting queries reuse that day's validated cache.
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
| `Ctrl+U` | Clear the filter text and remain in filter editing. |
| `Backspace` | Delete one character, or leave filter editing when already empty. |
| `Enter`, `Esc` | Finish filter editing. |
| `G`, End | Jump to the end. |
| `g`, Home | Jump to the start. |
| `a` | Show all access modes. |
| `h` | Show hybrid meetings only. |
| `p` | Show in-person-capable meetings, including hybrids. |
| `o` | Show online-capable meetings, including hybrids. |
| `r` | Restore the original command view at the top. |
| `q` | Quit from navigation mode. |

The live filter performs a case-insensitive match across every label and value
in a meeting table, including the inline access badge. A match retains the
entire meeting rather than selecting a single row, and every matching text
fragment is highlighted. While editing a filter, printable keys such as `a`,
`h`, `p`, `o`, and `q` remain search text. `Ctrl+U` clears the complete query
without leaving filter editing. Backspace on an empty query leaves filter
editing and returns to navigation.
Access shortcuts combine with the current text filter, and `a` restores every
access mode without clearing that text. The pager header shows the active
access filter. `--no-pager`, redirected stdin, or redirected stdout prints the
same complete tables directly.

Reset clears the pager's text query and access filter, leaves search-editing
mode, and returns the scroll position to zero. The original command filters,
query, result limit, ranking, and timing remain intact because reset does not
rerun or broaden the command-level query. While editing a text filter, `r`
remains ordinary search text.

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
