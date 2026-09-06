#![doc = include_str!("../README.md")]

use clap::{Command, CommandFactory, Parser};

#[derive(Parser)]
#[command(
    name = "nyintergroup",
    version,
    about = "Find New York Inter-Group AA meetings from the terminal"
)]
struct Cli;

/// Builds the complete `nyintergroup` command-line interface.
#[must_use]
pub fn command() -> Command {
    Cli::command()
}
