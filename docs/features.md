# Features

## Current behavior

The initial `nyintergroup` binary provides:

- `nyintergroup --help`;
- `nyintergroup --version`;
- the same help text when invoked without arguments.

It does not fetch or cache meeting data yet.

## Product direction

The CLI will query the New York Inter-Group meeting directory with basic
filters such as weekday, time, and meeting format. Finding a meeting available
now or soon is a first-class workflow, not a complicated combination of flags.

Online results should make joinability obvious. In-person results should expose
the listed address and a Google Maps directions link. Distance calculations are
optional and require a user-provided origin. Hybrid meetings retain both their
online and physical information.

The exact command surface, result ranking, definition of "now," output formats,
and data-access strategy remain intentionally undecided for the next product
discussion.

