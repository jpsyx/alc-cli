#![doc = include_str!("../README.md")]

use std::{ffi::OsString, fmt};

use chrono::{Datelike, NaiveDate};
use clap::{
    Args, Command, CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum, error::ErrorKind,
};

pub mod daily_reflection;
pub mod meeting;
mod schedule;
pub mod theme;

pub use schedule::{MeetingSchedule, ScheduleDay, ScheduleTime};

use daily_reflection::ReflectionDate;
use schedule::{MeetingScheduleArgs, MeetingShortcut};
use theme::Theme;

#[derive(Parser)]
#[command(
    name = "alc",
    version,
    disable_help_subcommand = true,
    about = "Find AA meetings and read Daily Reflections from the terminal",
    after_help = "Examples:\n  alc meeting\n  alc meeting find --weekday sunday --time morning --type online\n  alc daily\n  alc daily --date 09-06"
)]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand)]
enum CliCommand {
    /// Find meetings from the New York Inter-Group directory.
    #[command(
        after_help = "With no command, `alc meeting` runs `alc meeting now`. Schedule shortcuts include today, tomorrow, week (also `this week`), every weekday, morning, afternoon, and night (also evening). Date and time shortcuts can be combined in either order.\n\nExamples:\n  alc meeting\n  alc meeting now --type online --limit 10\n  alc meeting today --type online\n  alc meeting tomorrow night\n  alc meeting this week morning\n  alc meeting evening tuesday\n  alc meeting find \"turning point\"\n  alc meeting find --region brooklyn --type in-person"
    )]
    Meeting {
        #[command(subcommand)]
        command: Option<MeetingCommand>,
    },
    /// Read an AA Daily Reflection.
    #[command(
        visible_alias = "daily",
        after_help = "Defaults to today's local date. A date without a year uses the current local year.\n\nExamples:\n  alc daily\n  alc daily --date 09-06\n  alc daily-reflection --date 2024-02-29"
    )]
    DailyReflection {
        /// Reflection date as YYYY-MM-DD or MM-DD (default: today).
        #[arg(long, value_name = "DATE")]
        date: Option<String>,
    },
}

#[derive(Subcommand)]
enum MeetingCommand {
    /// Find recently started meetings or meetings starting soon.
    #[command(
        after_help = "By default, all active access types are included. Times use America/New_York. In-progress meetings show how long ago they started. In `now`, they are limited to meetings started within the last 30 minutes. Meetings beginning within the next hour are also included. Interactive terminals open the result pager unless --no-pager is set.\n\nPager keys: j/k or arrow keys move one line; d/u move half a page; / filters across every meeting field; G jumps to the end; r resets the original view; q quits. Access shortcuts: a shows all; h selects hybrid; p selects in-person; o selects online. Online and in-person include hybrid meetings. Filtering keeps each full meeting table and highlights matching text. While editing a filter, shortcut letters are search text, Ctrl+U clears the filter, and Backspace on an empty filter exits editing.\n\nExamples:\n  alc meeting now\n  alc meeting now --type online --limit 10\n  alc meeting now --type in-person --from \"10001\"\n  alc meeting now --no-pager"
    )]
    Now(MeetingNowArgs),
    /// Search and filter the complete meeting directory.
    #[command(
        after_help = "With no filters, searches all active meetings. Online-capable meetings appear first. In-progress meetings show how long ago they started. Interactive terminals open the result pager unless --no-pager is set.\n\nPager keys: j/k or arrow keys move one line; d/u move half a page; / filters across every meeting field; G jumps to the end; r resets the original view; q quits. Access shortcuts: a shows all; h selects hybrid; p selects in-person; o selects online. Online and in-person include hybrid meetings. Filtering keeps each full meeting table and highlights matching text. While editing a filter, shortcut letters are search text, Ctrl+U clears the filter, and Backspace on an empty filter exits editing.\n\nExamples:\n  alc meeting find\n  alc meeting find \"turning point\"\n  alc meeting find --weekday sunday --time morning --type online\n  alc meeting find --region brooklyn --type in-person --limit 10\n  alc meeting find --no-pager"
    )]
    Find(MeetingFindArgs),
    /// List meetings scheduled today.
    Today(MeetingScheduleArgs),
    /// List meetings scheduled tomorrow.
    Tomorrow(MeetingScheduleArgs),
    /// List meetings across the next seven days.
    Week(MeetingScheduleArgs),
    /// List Sunday meetings.
    Sunday(MeetingScheduleArgs),
    /// List Monday meetings.
    Monday(MeetingScheduleArgs),
    /// List Tuesday meetings.
    Tuesday(MeetingScheduleArgs),
    /// List Wednesday meetings.
    Wednesday(MeetingScheduleArgs),
    /// List Thursday meetings.
    Thursday(MeetingScheduleArgs),
    /// List Friday meetings.
    Friday(MeetingScheduleArgs),
    /// List Saturday meetings.
    Saturday(MeetingScheduleArgs),
    /// List this morning's meetings.
    Morning(MeetingScheduleArgs),
    /// List this afternoon's meetings.
    Afternoon(MeetingScheduleArgs),
    /// List tonight's meetings.
    #[command(visible_alias = "evening")]
    Night(MeetingScheduleArgs),
}

#[derive(Args)]
struct MeetingNowArgs {
    /// Restrict results by access type.
    #[arg(long = "type", value_enum)]
    attendance: Option<Attendance>,
    #[command(flatten)]
    results: ResultArgs,
}

#[derive(Args)]
struct MeetingFindArgs {
    /// Search meeting, group, location, address, and region text.
    #[arg(value_name = "QUERY")]
    query: Option<String>,
    /// Restrict results to one weekday.
    #[arg(long, value_enum)]
    weekday: Option<Weekday>,
    /// Restrict results to a time of day.
    #[arg(long, value_enum)]
    time: Option<TimeOfDay>,
    /// Restrict results by access type.
    #[arg(long = "type", value_enum)]
    attendance: Option<Attendance>,
    /// Restrict results to regions containing this text.
    #[arg(long, value_name = "TEXT")]
    region: Option<String>,
    #[command(flatten)]
    results: ResultArgs,
}

#[derive(Args)]
struct ResultArgs {
    /// Maximum number of results to print.
    #[arg(long, default_value_t = 20, value_parser = clap::value_parser!(u16).range(1..))]
    limit: u16,
    /// Starting address to include in Google Maps direction links.
    #[arg(long = "from", value_name = "ADDRESS")]
    origin: Option<String>,
    /// Print full meeting tables directly instead of opening the pager.
    #[arg(long)]
    no_pager: bool,
}

/// The action requested through the command line.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Request {
    /// Query New York Inter-Group meetings.
    Meeting(MeetingRequest),
    /// Display the reflection for the validated date.
    DailyReflection(ReflectionDate),
}

/// A meeting-directory action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MeetingRequest {
    /// Find recently started meetings or meetings starting soon.
    Now(MeetingNow),
    /// Search the directory with explicit filters.
    Find(MeetingFilters),
    /// List meetings using composable date and time-of-day shortcuts.
    Schedule(MeetingSchedule),
}

/// Options for a meeting-now query.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeetingNow {
    attendance: Option<Attendance>,
    limit: usize,
    origin: Option<String>,
    pager_enabled: bool,
}

impl MeetingNow {
    /// Returns the requested access filter.
    #[must_use]
    pub const fn attendance(&self) -> Option<Attendance> {
        self.attendance
    }

    /// Returns the maximum number of results.
    #[must_use]
    pub const fn limit(&self) -> usize {
        self.limit
    }

    /// Returns an optional starting address for direction links.
    #[must_use]
    pub fn origin(&self) -> Option<&str> {
        self.origin.as_deref()
    }

    /// Returns whether an interactive terminal may open the result pager.
    #[must_use]
    pub const fn pager_enabled(&self) -> bool {
        self.pager_enabled
    }
}

/// Filters for a meeting-directory search.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeetingFilters {
    query: Option<String>,
    weekday: Option<Weekday>,
    time: Option<TimeOfDay>,
    attendance: Option<Attendance>,
    region: Option<String>,
    limit: usize,
    origin: Option<String>,
    pager_enabled: bool,
}

impl MeetingFilters {
    /// Returns the free-text query.
    #[must_use]
    pub fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }

    /// Returns the weekday filter.
    #[must_use]
    pub const fn weekday(&self) -> Option<Weekday> {
        self.weekday
    }

    /// Returns the time-of-day filter.
    #[must_use]
    pub const fn time(&self) -> Option<TimeOfDay> {
        self.time
    }

    /// Returns the access filter.
    #[must_use]
    pub const fn attendance(&self) -> Option<Attendance> {
        self.attendance
    }

    /// Returns the region-text filter.
    #[must_use]
    pub fn region(&self) -> Option<&str> {
        self.region.as_deref()
    }

    /// Returns the maximum number of results.
    #[must_use]
    pub const fn limit(&self) -> usize {
        self.limit
    }

    /// Returns an optional starting address for direction links.
    #[must_use]
    pub fn origin(&self) -> Option<&str> {
        self.origin.as_deref()
    }

    /// Returns whether an interactive terminal may open the result pager.
    #[must_use]
    pub const fn pager_enabled(&self) -> bool {
        self.pager_enabled
    }
}

/// An access capability used to filter meetings.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum Attendance {
    /// Meetings with a conference URL or phone number, including hybrids.
    Online,
    /// Meetings with a physical location, including hybrids.
    InPerson,
    /// Meetings offering both online and in-person access.
    Hybrid,
}

impl Attendance {
    pub(crate) const fn matches(self, is_online: bool, is_in_person: bool) -> bool {
        match self {
            Self::Online => is_online,
            Self::InPerson => is_in_person,
            Self::Hybrid => is_online && is_in_person,
        }
    }
}

impl fmt::Display for Attendance {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Online => formatter.write_str("online"),
            Self::InPerson => formatter.write_str("in-person"),
            Self::Hybrid => formatter.write_str("hybrid"),
        }
    }
}

/// A weekday in the NYIG directory's Sunday-first convention.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum Weekday {
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
}

impl fmt::Display for Weekday {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Sunday => "sunday",
            Self::Monday => "monday",
            Self::Tuesday => "tuesday",
            Self::Wednesday => "wednesday",
            Self::Thursday => "thursday",
            Self::Friday => "friday",
            Self::Saturday => "saturday",
        };
        formatter.write_str(value)
    }
}

impl Weekday {
    pub(crate) const fn directory_index(self) -> u8 {
        match self {
            Self::Sunday => 0,
            Self::Monday => 1,
            Self::Tuesday => 2,
            Self::Wednesday => 3,
            Self::Thursday => 4,
            Self::Friday => 5,
            Self::Saturday => 6,
        }
    }
}

/// A time-of-day category matching the NYIG directory.
#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum TimeOfDay {
    Morning,
    Midday,
    Evening,
    Night,
}

impl fmt::Display for TimeOfDay {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Morning => formatter.write_str("morning"),
            Self::Midday => formatter.write_str("midday"),
            Self::Evening => formatter.write_str("evening"),
            Self::Night => formatter.write_str("night"),
        }
    }
}

impl TimeOfDay {
    pub(crate) const fn contains_minutes(self, minutes: u32) -> bool {
        match self {
            Self::Morning => minutes >= 240 && minutes < 720,
            Self::Midday => minutes >= 660 && minutes < 1_020,
            Self::Evening => minutes >= 960 && minutes < 1_260,
            Self::Night => minutes >= 1_200 || minutes < 300,
        }
    }
}

impl fmt::Display for Request {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Meeting(MeetingRequest::Now(_)) => formatter.write_str("meeting:now"),
            Self::Meeting(MeetingRequest::Find(_)) => formatter.write_str("meeting:find"),
            Self::Meeting(MeetingRequest::Schedule(options)) => {
                write!(formatter, "meeting:schedule:{}", options.day())?;
                if let Some(time) = options.time() {
                    write!(formatter, ":{time}")?;
                }
                Ok(())
            }
            Self::DailyReflection(date) => write!(formatter, "daily-reflection:{date}"),
        }
    }
}

/// Builds the complete `alc` command-line interface.
#[must_use]
pub fn command() -> Command {
    command_with_theme(Theme::active())
}

/// Builds the complete command-line interface with an explicit terminal theme.
#[must_use]
pub fn command_with_theme(theme: Theme) -> Command {
    let mut command = Cli::command();
    schedule::add_help(&mut command);
    command
        .styles(theme.clap_styles())
        .color(theme.clap_color_choice())
}

/// Parses command-line arguments into a validated request.
pub fn parse_request<I, T>(args: I, today: NaiveDate) -> Result<Request, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    parse_request_with_today(args, || today)
}

/// Parses command-line arguments and reads the current date only when needed.
///
/// Help, version, meeting commands, and argument errors are resolved without
/// invoking `today`.
pub fn parse_request_with_today<I, T, F>(args: I, today: F) -> Result<Request, clap::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
    F: FnOnce() -> NaiveDate,
{
    let matches = command().try_get_matches_from(schedule::normalize_this_week_alias(args))?;
    let cli = Cli::from_arg_matches(&matches)?;

    match cli.command {
        CliCommand::Meeting { command } => parse_meeting_command(command).map(Request::Meeting),
        CliCommand::DailyReflection { date } => {
            let today = today();
            let date = match date {
                Some(input) => ReflectionDate::parse(&input, today.year())
                    .map_err(|error| Cli::command().error(ErrorKind::ValueValidation, error))?,
                None => ReflectionDate::from(today),
            };
            Ok(Request::DailyReflection(date))
        }
    }
}

fn parse_meeting_command(command: Option<MeetingCommand>) -> Result<MeetingRequest, clap::Error> {
    match command {
        None => Ok(MeetingRequest::Now(MeetingNow {
            attendance: None,
            limit: 20,
            origin: None,
            pager_enabled: true,
        })),
        Some(MeetingCommand::Now(args)) => Ok(MeetingRequest::Now(MeetingNow {
            attendance: args.attendance,
            limit: usize::from(args.results.limit),
            origin: args.results.origin,
            pager_enabled: !args.results.no_pager,
        })),
        Some(MeetingCommand::Find(args)) => Ok(MeetingRequest::Find(MeetingFilters {
            query: args.query,
            weekday: args.weekday,
            time: args.time,
            attendance: args.attendance,
            region: args.region,
            limit: usize::from(args.results.limit),
            origin: args.results.origin,
            pager_enabled: !args.results.no_pager,
        })),
        Some(MeetingCommand::Today(args)) => parse_schedule(MeetingShortcut::Today, args),
        Some(MeetingCommand::Tomorrow(args)) => parse_schedule(MeetingShortcut::Tomorrow, args),
        Some(MeetingCommand::Week(args)) => parse_schedule(MeetingShortcut::Week, args),
        Some(MeetingCommand::Sunday(args)) => parse_schedule(MeetingShortcut::Sunday, args),
        Some(MeetingCommand::Monday(args)) => parse_schedule(MeetingShortcut::Monday, args),
        Some(MeetingCommand::Tuesday(args)) => parse_schedule(MeetingShortcut::Tuesday, args),
        Some(MeetingCommand::Wednesday(args)) => parse_schedule(MeetingShortcut::Wednesday, args),
        Some(MeetingCommand::Thursday(args)) => parse_schedule(MeetingShortcut::Thursday, args),
        Some(MeetingCommand::Friday(args)) => parse_schedule(MeetingShortcut::Friday, args),
        Some(MeetingCommand::Saturday(args)) => parse_schedule(MeetingShortcut::Saturday, args),
        Some(MeetingCommand::Morning(args)) => parse_schedule(MeetingShortcut::Morning, args),
        Some(MeetingCommand::Afternoon(args)) => parse_schedule(MeetingShortcut::Afternoon, args),
        Some(MeetingCommand::Night(args)) => parse_schedule(MeetingShortcut::Night, args),
    }
}

fn parse_schedule(
    first: MeetingShortcut,
    args: MeetingScheduleArgs,
) -> Result<MeetingRequest, clap::Error> {
    schedule::parse(first, args)
        .map(MeetingRequest::Schedule)
        .map_err(|message| command().error(ErrorKind::ArgumentConflict, message))
}
