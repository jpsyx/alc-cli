use std::{cmp::Ordering, fmt};

use chrono::{DateTime, Datelike, Timelike};
use chrono_tz::Tz;

use super::model::Meeting;
use crate::{MeetingFilters, MeetingNow, MeetingSchedule, ScheduleDay};

const MINUTES_PER_DAY: i64 = 1_440;
const MINUTES_PER_WEEK: i64 = 10_080;
const NOW_HORIZON_MINUTES: i64 = 60;
const NOW_MAX_ELAPSED_MINUTES: i64 = 30;

/// Filters the active directory and ranks online-capable meetings first.
#[must_use]
pub fn find<'a>(meetings: &'a [Meeting], filters: &MeetingFilters) -> Vec<&'a Meeting> {
    let query = normalized(filters.query());
    let region = normalized(filters.region());
    let mut matches = meetings
        .iter()
        .filter(|meeting| meeting.is_active())
        .filter(|meeting| {
            filters.attendance().is_none_or(|attendance| {
                attendance.matches(meeting.is_online(), meeting.is_in_person())
            })
        })
        .filter(|meeting| {
            filters
                .weekday()
                .is_none_or(|weekday| weekday.directory_index() == meeting.day)
        })
        .filter(|meeting| {
            filters.time().is_none_or(|time| {
                time.contains_minutes(meeting.start.num_seconds_from_midnight() / 60)
            })
        })
        .filter(|meeting| {
            query
                .as_deref()
                .is_none_or(|query| meeting.search_text().contains(query))
        })
        .filter(|meeting| {
            region
                .as_deref()
                .is_none_or(|region| meeting.region_text().contains(region))
        })
        .collect::<Vec<_>>();

    matches.sort_by(|left, right| meeting_order(left, right));
    matches.truncate(filters.limit());
    matches
}

/// Filters meetings using a relative date or weekday and optional time shorthand.
#[must_use]
pub fn schedule<'a>(
    meetings: &'a [Meeting],
    at: &DateTime<Tz>,
    options: &MeetingSchedule,
) -> Vec<&'a Meeting> {
    let today = u8::try_from(at.weekday().num_days_from_sunday())
        .expect("a weekday index should fit in u8");
    let region = normalized(options.region());
    let mut matches = meetings
        .iter()
        .filter(|meeting| meeting.is_active())
        .filter(|meeting| schedule_day_matches(options.day(), meeting.day, today))
        .filter(|meeting| {
            options.time().is_none_or(|time| {
                time.contains_minutes(meeting.start.num_seconds_from_midnight() / 60)
            })
        })
        .filter(|meeting| {
            options.attendance().is_none_or(|attendance| {
                attendance.matches(meeting.is_online(), meeting.is_in_person())
            })
        })
        .filter(|meeting| {
            region
                .as_deref()
                .is_none_or(|region| meeting.region_text().contains(region))
        })
        .collect::<Vec<_>>();

    matches.sort_by(|left, right| {
        online_rank(left)
            .cmp(&online_rank(right))
            .then_with(|| {
                schedule_day_offset(options.day(), left.day, today).cmp(&schedule_day_offset(
                    options.day(),
                    right.day,
                    today,
                ))
            })
            .then_with(|| left.start.cmp(&right.start))
            .then_with(|| left.name.cmp(&right.name))
    });
    matches.truncate(options.limit());
    matches
}

const fn schedule_day_matches(scope: ScheduleDay, meeting_day: u8, today: u8) -> bool {
    match scope {
        ScheduleDay::Today => meeting_day == today,
        ScheduleDay::Tomorrow => meeting_day == (today + 1) % 7,
        ScheduleDay::Week => true,
        ScheduleDay::Weekday(weekday) => meeting_day == weekday.directory_index(),
    }
}

const fn schedule_day_offset(scope: ScheduleDay, meeting_day: u8, today: u8) -> u8 {
    if matches!(scope, ScheduleDay::Week) {
        (meeting_day + 7 - today) % 7
    } else {
        0
    }
}

/// A meeting returned by a current-availability query.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AvailableMeeting<'a> {
    pub(super) meeting: &'a Meeting,
    pub(super) availability: Availability,
    minutes_until: i64,
}

impl<'a> AvailableMeeting<'a> {
    /// Returns the matching meeting.
    #[must_use]
    pub const fn meeting(&self) -> &'a Meeting {
        self.meeting
    }

    /// Returns whether the meeting is underway or how soon it starts.
    #[must_use]
    pub const fn availability(&self) -> Availability {
        self.availability
    }
}

/// Current timing for a meeting returned by [`now`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Availability {
    /// The scheduled meeting is underway by the contained number of minutes.
    InProgress(i64),
    /// The meeting begins after the contained number of minutes.
    StartsIn(i64),
}

impl fmt::Display for Availability {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InProgress(_) => formatter.write_str("in progress"),
            Self::StartsIn(0) => formatter.write_str("starts now"),
            Self::StartsIn(minutes) => write!(formatter, "starts in {minutes}m"),
        }
    }
}

/// Finds meetings started within 30 minutes or starting within the next hour.
#[must_use]
pub fn now<'a>(
    meetings: &'a [Meeting],
    at: DateTime<Tz>,
    options: &MeetingNow,
) -> Vec<AvailableMeeting<'a>> {
    let now_minutes = week_minutes(&at);
    let mut matches = meetings
        .iter()
        .filter(|meeting| meeting.is_active())
        .filter(|meeting| {
            options.attendance().is_none_or(|attendance| {
                attendance.matches(meeting.is_online(), meeting.is_in_person())
            })
        })
        .filter_map(|meeting| available_meeting(meeting, now_minutes, NOW_HORIZON_MINUTES))
        .filter(|available| within_now_window(available.availability))
        .collect::<Vec<_>>();

    matches.sort_by(|left, right| {
        online_rank(left.meeting)
            .cmp(&online_rank(right.meeting))
            .then_with(|| {
                availability_rank(left.availability).cmp(&availability_rank(right.availability))
            })
            .then_with(|| left.minutes_until.cmp(&right.minutes_until))
            .then_with(|| left.meeting.name.cmp(&right.meeting.name))
    });
    matches.truncate(options.limit());
    matches
}

const fn within_now_window(availability: Availability) -> bool {
    match availability {
        Availability::InProgress(minutes) => minutes <= NOW_MAX_ELAPSED_MINUTES,
        Availability::StartsIn(_) => true,
    }
}

pub(super) fn availability_within(
    meeting: &Meeting,
    at: &DateTime<Tz>,
    horizon_minutes: i64,
) -> Option<Availability> {
    available_meeting(meeting, week_minutes(at), horizon_minutes)
        .map(|available| available.availability)
}

fn week_minutes(at: &DateTime<Tz>) -> i64 {
    i64::from(at.weekday().num_days_from_sunday()) * MINUTES_PER_DAY
        + i64::from(at.hour() * 60 + at.minute())
}

fn normalized(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_lowercase)
}

fn meeting_order(left: &Meeting, right: &Meeting) -> Ordering {
    online_rank(left)
        .cmp(&online_rank(right))
        .then_with(|| left.day.cmp(&right.day))
        .then_with(|| left.start.cmp(&right.start))
        .then_with(|| left.name.cmp(&right.name))
}

const fn online_rank(meeting: &Meeting) -> u8 {
    if meeting.is_online() { 0 } else { 1 }
}

const fn availability_rank(availability: Availability) -> u8 {
    match availability {
        Availability::InProgress(_) => 0,
        Availability::StartsIn(_) => 1,
    }
}

fn available_meeting(
    meeting: &Meeting,
    now_minutes: i64,
    horizon_minutes: i64,
) -> Option<AvailableMeeting<'_>> {
    let start = i64::from(meeting.day) * MINUTES_PER_DAY + meeting.start_minutes();
    let mut duration = meeting.end_minutes() - meeting.start_minutes();
    if duration <= 0 {
        duration += MINUTES_PER_DAY;
    }
    let elapsed = (now_minutes - start).rem_euclid(MINUTES_PER_WEEK);
    if elapsed < duration {
        return Some(AvailableMeeting {
            meeting,
            availability: Availability::InProgress(elapsed),
            minutes_until: 0,
        });
    }

    let minutes_until = (start - now_minutes).rem_euclid(MINUTES_PER_WEEK);
    (minutes_until <= horizon_minutes).then_some(AvailableMeeting {
        meeting,
        availability: Availability::StartsIn(minutes_until),
        minutes_until,
    })
}
