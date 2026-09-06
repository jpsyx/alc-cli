use std::{ffi::OsString, fmt};

use clap::{Args, Command, ValueEnum};

use crate::{Attendance, ResultArgs, TimeOfDay, Weekday};

const COMMANDS: [&str; 13] = [
    "today",
    "tomorrow",
    "week",
    "sunday",
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
    "morning",
    "afternoon",
    "night",
];
const AFTER_HELP: &str = "Date and time shortcuts can follow in either order. `this week` aliases `week`, and `evening` aliases `night`. A time without a date means today. The 30-minute in-progress cutoff applies only to `alc meeting now` and bare `alc meeting`. Interactive terminals open the result pager unless --no-pager is set.\n\nPager keys: j/k or arrow keys move one line; d/u move half a page; / filters across every meeting field; G jumps to the end; r resets the original view; q quits. Access shortcuts: a shows all; h selects hybrid; p selects in-person; o selects online. Online and in-person include hybrid meetings. Filtering keeps each full meeting table and highlights matching text. While editing a filter, shortcut letters are search text, Ctrl+U clears the filter, and Backspace on an empty filter exits editing.\n\nExamples:\n  alc meeting today --type online\n  alc meeting tomorrow night\n  alc meeting this week morning\n  alc meeting evening tuesday --region brooklyn";

#[derive(Args)]
pub struct MeetingScheduleArgs {
    /// Additional date or time-of-day shorthand filters.
    #[arg(value_name = "FILTER", value_enum)]
    filters: Vec<MeetingShortcut>,
    /// Restrict results by access type.
    #[arg(long = "type", value_enum)]
    attendance: Option<Attendance>,
    /// Restrict results to regions containing this text.
    #[arg(long, value_name = "TEXT")]
    region: Option<String>,
    #[command(flatten)]
    results: ResultArgs,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum MeetingShortcut {
    Today,
    Tomorrow,
    Week,
    Sunday,
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Morning,
    Afternoon,
    Night,
    Evening,
}

/// Options for a date or time-of-day meeting schedule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeetingSchedule {
    day: ScheduleDay,
    time: Option<ScheduleTime>,
    attendance: Option<Attendance>,
    region: Option<String>,
    limit: usize,
    origin: Option<String>,
    pager_enabled: bool,
}

impl MeetingSchedule {
    /// Returns the selected date scope.
    #[must_use]
    pub const fn day(&self) -> ScheduleDay {
        self.day
    }

    /// Returns the optional time-of-day filter.
    #[must_use]
    pub const fn time(&self) -> Option<ScheduleTime> {
        self.time
    }

    /// Returns the requested access filter.
    #[must_use]
    pub const fn attendance(&self) -> Option<Attendance> {
        self.attendance
    }

    /// Returns the optional region-text filter.
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

/// A date scope for a meeting schedule shortcut.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduleDay {
    /// Meetings on the current New York date.
    Today,
    /// Meetings on the next New York date.
    Tomorrow,
    /// Meetings across the next seven days, beginning today.
    Week,
    /// Meetings on a named weekday.
    Weekday(Weekday),
}

impl fmt::Display for ScheduleDay {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Today => formatter.write_str("today"),
            Self::Tomorrow => formatter.write_str("tomorrow"),
            Self::Week => formatter.write_str("week"),
            Self::Weekday(weekday) => weekday.fmt(formatter),
        }
    }
}

/// A user-friendly time range for schedule shortcuts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduleTime {
    /// The NYIG morning range.
    Morning,
    /// The NYIG midday range, presented as afternoon.
    Afternoon,
    /// The combined NYIG evening and night ranges.
    Night,
}

impl fmt::Display for ScheduleTime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Morning => formatter.write_str("morning"),
            Self::Afternoon => formatter.write_str("afternoon"),
            Self::Night => formatter.write_str("night"),
        }
    }
}

impl ScheduleTime {
    pub(crate) const fn contains_minutes(self, minutes: u32) -> bool {
        match self {
            Self::Morning => TimeOfDay::Morning.contains_minutes(minutes),
            Self::Afternoon => TimeOfDay::Midday.contains_minutes(minutes),
            Self::Night => {
                TimeOfDay::Evening.contains_minutes(minutes)
                    || TimeOfDay::Night.contains_minutes(minutes)
            }
        }
    }
}

pub fn add_help(command: &mut Command) {
    if let Some(meeting) = command.find_subcommand_mut("meeting") {
        for name in COMMANDS {
            if let Some(schedule) = meeting.find_subcommand_mut(name) {
                *schedule = schedule.clone().after_help(AFTER_HELP);
            }
        }
    }
}

pub fn normalize_this_week_alias<I, T>(args: I) -> Vec<OsString>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString>,
{
    let args = args.into_iter().map(Into::into).collect::<Vec<_>>();
    if args.get(1).is_none_or(|argument| argument != "meeting")
        || args
            .get(2)
            .is_some_and(|argument| argument == "now" || argument == "find")
    {
        return args;
    }

    let mut normalized = Vec::with_capacity(args.len());
    let mut index = 0;
    while index < args.len() {
        if index >= 2
            && args[index] == "this"
            && args
                .get(index + 1)
                .is_some_and(|argument| argument == "week")
        {
            normalized.push(OsString::from("week"));
            index += 2;
        } else {
            normalized.push(args[index].clone());
            index += 1;
        }
    }
    normalized
}

pub fn parse(
    first: MeetingShortcut,
    args: MeetingScheduleArgs,
) -> Result<MeetingSchedule, &'static str> {
    let mut day = None;
    let mut time = None;
    for shortcut in std::iter::once(first).chain(args.filters) {
        let (next_day, next_time) = shortcut.schedule_filter();
        if let Some(next_day) = next_day {
            if day.is_some_and(|day| day != next_day) {
                return Err("choose only one date or weekday shortcut");
            }
            day = Some(next_day);
        }
        if let Some(next_time) = next_time {
            if time.is_some_and(|time| time != next_time) {
                return Err("choose only one time-of-day shortcut");
            }
            time = Some(next_time);
        }
    }

    Ok(MeetingSchedule {
        day: day.unwrap_or(ScheduleDay::Today),
        time,
        attendance: args.attendance,
        region: args.region,
        limit: usize::from(args.results.limit),
        origin: args.results.origin,
        pager_enabled: !args.results.no_pager,
    })
}

impl MeetingShortcut {
    const fn schedule_filter(self) -> (Option<ScheduleDay>, Option<ScheduleTime>) {
        match self {
            Self::Today => (Some(ScheduleDay::Today), None),
            Self::Tomorrow => (Some(ScheduleDay::Tomorrow), None),
            Self::Week => (Some(ScheduleDay::Week), None),
            Self::Sunday => (Some(ScheduleDay::Weekday(Weekday::Sunday)), None),
            Self::Monday => (Some(ScheduleDay::Weekday(Weekday::Monday)), None),
            Self::Tuesday => (Some(ScheduleDay::Weekday(Weekday::Tuesday)), None),
            Self::Wednesday => (Some(ScheduleDay::Weekday(Weekday::Wednesday)), None),
            Self::Thursday => (Some(ScheduleDay::Weekday(Weekday::Thursday)), None),
            Self::Friday => (Some(ScheduleDay::Weekday(Weekday::Friday)), None),
            Self::Saturday => (Some(ScheduleDay::Weekday(Weekday::Saturday)), None),
            Self::Morning => (None, Some(ScheduleTime::Morning)),
            Self::Afternoon => (None, Some(ScheduleTime::Afternoon)),
            Self::Night | Self::Evening => (None, Some(ScheduleTime::Night)),
        }
    }
}
