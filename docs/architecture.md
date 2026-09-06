# Architecture

## Current shape

The crate is intentionally small:

- `src/lib.rs` owns the Clap command definition so command metadata is reusable
  and testable;
- `src/main.rs` is a thin process shell that prints help for a bare invocation
  and delegates argument handling to Clap;
- `tests/cli.rs` verifies the compiled binary's public help and version contract.

The sole runtime dependency is Clap. No HTTP client, HTML parser, async runtime,
cache, time library, geocoder, or terminal styling library has been selected.

## Expected seams

Meeting functionality should preserve separate boundaries for source retrieval,
source parsing, normalized meeting data, pure filtering and ranking, time and
location context, and output rendering. This direction does not prescribe one
module per boundary before behavior requires it.

The upstream directory is mutable external input. Retrieval and parsing errors
must remain distinguishable from an authoritative empty result.

