use chrono::{NaiveTime, Timelike};
use url::Url;

use crate::theme::Theme;

use super::{
    model::{Access, Meeting},
    query::{Availability, AvailableMeeting},
    table::{border, case_insensitive_match_ranges, contains_case_insensitive, pad, wrap_text},
};

const TABLE_WIDTH: usize = 120;
const LABEL_WIDTH: usize = 14;

/// Creates a Google Maps directions URL for an in-person meeting.
#[must_use]
pub fn directions_url(meeting: &Meeting, origin: Option<&str>) -> Option<String> {
    if !meeting.is_in_person() {
        return None;
    }
    let destination = meeting.address.as_deref()?.trim();
    if destination.is_empty() {
        return None;
    }

    let mut url = Url::parse("https://www.google.com/maps/dir/")
        .expect("the fixed Google Maps URL should be valid");
    {
        let mut query = url.query_pairs_mut();
        query.append_pair("api", "1");
        if let Some(origin) = origin.filter(|value| !value.trim().is_empty()) {
            query.append_pair("origin", origin);
        }
        query.append_pair("destination", destination);
    }
    Some(url.to_string())
}

/// Renders filtered meetings for human-readable terminal output.
#[must_use]
pub fn render_find(meetings: &[&Meeting], origin: Option<&str>, theme: Theme) -> String {
    find_list(meetings, origin).render_filtered("", TABLE_WIDTH, theme)
}

pub(super) fn find_list(meetings: &[&Meeting], origin: Option<&str>) -> MeetingList {
    let noun = if meetings.len() == 1 {
        "meeting"
    } else {
        "meetings"
    };
    MeetingList {
        heading: format!("{} {noun} found (online-capable first).", meetings.len()),
        empty: "No meetings found. Try broader filters.".to_owned(),
        tables: meetings
            .iter()
            .map(|meeting| meeting_table(meeting, None, origin))
            .collect(),
    }
}

/// Renders current and upcoming meetings for human-readable terminal output.
#[must_use]
pub fn render_now(meetings: &[AvailableMeeting<'_>], origin: Option<&str>, theme: Theme) -> String {
    now_list(meetings, origin).render_filtered("", TABLE_WIDTH, theme)
}

pub(super) fn now_list(meetings: &[AvailableMeeting<'_>], origin: Option<&str>) -> MeetingList {
    let noun = if meetings.len() == 1 {
        "meeting"
    } else {
        "meetings"
    };
    MeetingList {
        heading: format!("{} {noun} available now (New York time).", meetings.len()),
        empty: "No meetings are in progress or starting within the next hour.".to_owned(),
        tables: meetings
            .iter()
            .map(|item| meeting_table(item.meeting, Some(item.availability), origin))
            .collect(),
    }
}

pub(super) struct MeetingList {
    heading: String,
    empty: String,
    tables: Vec<MeetingTable>,
}

impl MeetingList {
    fn matching(&self, query: &str) -> Vec<&MeetingTable> {
        self.tables
            .iter()
            .filter(|table| table.matches(query))
            .collect()
    }

    pub(super) fn matching_count(&self, query: &str) -> usize {
        self.matching(query).len()
    }

    pub(super) fn total_count(&self) -> usize {
        self.tables.len()
    }

    pub(super) fn rendered_lines(&self, query: &str, width: usize, theme: Theme) -> Vec<String> {
        self.render_filtered(query, width, theme)
            .lines()
            .map(str::to_owned)
            .collect()
    }

    pub(super) fn render_static(&self, theme: Theme) -> String {
        self.render_filtered("", TABLE_WIDTH, theme)
    }

    fn render_filtered(&self, query: &str, width: usize, theme: Theme) -> String {
        let matching = self.matching(query);
        if matching.is_empty() {
            let message = if query.is_empty() {
                self.empty.clone()
            } else {
                format!("No meetings match \"{query}\".")
            };
            let message = wrap_text(&message, width)
                .iter()
                .map(|line| theme.warning(line))
                .collect::<Vec<_>>()
                .join("\n");
            return format!("{message}\n");
        }

        let heading = if query.is_empty() {
            self.heading.clone()
        } else {
            format!(
                "{} of {} meetings match \"{query}\".",
                matching.len(),
                self.tables.len()
            )
        };
        let heading = wrap_text(&heading, width)
            .iter()
            .map(|line| theme.heading(line))
            .collect::<Vec<_>>()
            .join("\n");
        let blocks = matching
            .iter()
            .map(|table| table.render(width, query, theme))
            .collect::<Vec<_>>();
        format!("{heading}\n\n{}\n", blocks.join("\n\n"))
    }
}

struct MeetingTable {
    fields: Vec<MeetingField>,
}

impl MeetingTable {
    fn matches(&self, query: &str) -> bool {
        query.is_empty()
            || self.fields.iter().any(|field| {
                contains_case_insensitive(field.label, query)
                    || contains_case_insensitive(&field.value, query)
            })
    }

    fn render(&self, width: usize, query: &str, theme: Theme) -> String {
        let table_width = width.max(LABEL_WIDTH + 8);
        let value_width = table_width - LABEL_WIDTH - 7;
        let top = border('┌', '┬', '┐', LABEL_WIDTH, value_width, theme);
        let middle = border('├', '┼', '┤', LABEL_WIDTH, value_width, theme);
        let bottom = border('└', '┴', '┘', LABEL_WIDTH, value_width, theme);
        let mut lines = vec![top];

        for (field_index, field) in self.fields.iter().enumerate() {
            let matches = case_insensitive_match_ranges(&field.value, query);
            let mut value_cursor = 0;
            for (line_index, value) in wrap_text(&field.value, value_width).iter().enumerate() {
                let label = if line_index == 0 { field.label } else { "" };
                let value_start = field.value[value_cursor..]
                    .find(value)
                    .map_or(value_cursor, |start| value_cursor + start);
                let value_end = value_start + value.len();
                let line_matches = intersecting_ranges(&matches, value_start, value_end);
                value_cursor = value_end;
                lines.push(format!(
                    "{} {} {} {} {}",
                    theme.muted("│"),
                    highlight_matches(&pad(label, LABEL_WIDTH), query, FieldStyle::Label, theme),
                    theme.muted("│"),
                    highlight_ranges(&pad(value, value_width), &line_matches, field.style, theme),
                    theme.muted("│")
                ));
            }
            if field_index + 1 < self.fields.len() {
                lines.push(middle.clone());
            }
        }
        lines.push(bottom);
        lines.join("\n")
    }
}

fn meeting_table(
    meeting: &Meeting,
    availability: Option<Availability>,
    origin: Option<&str>,
) -> MeetingTable {
    let mut fields = vec![
        MeetingField::primary("Meeting", &meeting.name),
        MeetingField::access("Access", access_label(meeting), meeting.access),
        MeetingField::secondary(
            "Schedule",
            format!(
                "{} {} to {}",
                weekday_name(meeting.day),
                format_clock(meeting.start),
                format_clock(meeting.end)
            ),
        ),
    ];
    if let Some(availability) = availability {
        fields.push(MeetingField::status("Status", availability.to_string()));
    }
    if let Some(group) = meeting
        .group
        .as_deref()
        .filter(|group| !group.eq_ignore_ascii_case(&meeting.name))
    {
        fields.push(MeetingField::primary("Group", group));
    }
    if let Some(join_url) = meeting.conference_url.as_deref() {
        fields.push(MeetingField::primary("Join", join_url));
    }
    if let Some(notes) = meeting.conference_url_notes.as_deref() {
        fields.push(MeetingField::secondary("Online details", one_line(notes)));
    }
    if let Some(phone) = meeting.conference_phone.as_deref() {
        fields.push(MeetingField::primary("Phone", phone));
    }
    if let Some(notes) = meeting.conference_phone_notes.as_deref() {
        fields.push(MeetingField::secondary("Phone details", one_line(notes)));
    }
    if meeting.is_in_person() {
        if let Some(location) = meeting.location.as_deref() {
            fields.push(MeetingField::primary("Place", location));
        }
        if let Some(address) = meeting.address.as_deref() {
            fields.push(MeetingField::primary("Address", address));
        }
        if let Some(url) = directions_url(meeting, origin) {
            fields.push(MeetingField::primary("Directions", url));
        }
    }
    if !meeting.regions.is_empty() {
        fields.push(MeetingField::secondary(
            "Region",
            meeting.regions.join(" > "),
        ));
    }
    if !meeting.types.is_empty() {
        fields.push(MeetingField::secondary("Types", meeting.types.join(", ")));
    }
    fields.push(MeetingField::primary("Source", &meeting.source_url));

    MeetingTable { fields }
}

struct MeetingField {
    label: &'static str,
    value: String,
    style: FieldStyle,
}

impl MeetingField {
    fn primary(label: &'static str, value: impl Into<String>) -> Self {
        Self::styled(label, value, FieldStyle::Primary)
    }

    fn secondary(label: &'static str, value: impl Into<String>) -> Self {
        Self::styled(label, value, FieldStyle::Secondary)
    }

    fn access(label: &'static str, value: impl Into<String>, access: Access) -> Self {
        Self::styled(label, value, FieldStyle::Access(access))
    }

    fn status(label: &'static str, value: impl Into<String>) -> Self {
        Self::styled(label, value, FieldStyle::Status)
    }

    fn styled(label: &'static str, value: impl Into<String>, style: FieldStyle) -> Self {
        let value = value.into();
        Self {
            label,
            value: one_line(&value),
            style,
        }
    }
}

#[derive(Clone, Copy)]
enum FieldStyle {
    Label,
    Primary,
    Secondary,
    Access(Access),
    Status,
}

fn style_value(value: &str, style: FieldStyle, theme: Theme) -> String {
    match style {
        FieldStyle::Label | FieldStyle::Access(Access::InPerson) => theme.accent(value),
        FieldStyle::Primary => theme.value(value),
        FieldStyle::Secondary | FieldStyle::Access(Access::Inactive) => theme.muted(value),
        FieldStyle::Access(Access::Online) | FieldStyle::Status => theme.success(value),
        FieldStyle::Access(Access::Hybrid) => theme.info(value),
    }
}

fn highlight_matches(value: &str, query: &str, style: FieldStyle, theme: Theme) -> String {
    if query.is_empty() {
        return style_value(value, style, theme);
    }

    let ranges = case_insensitive_match_ranges(value, query);
    highlight_ranges(value, &ranges, style, theme)
}

fn highlight_ranges(
    value: &str,
    ranges: &[std::ops::Range<usize>],
    style: FieldStyle,
    theme: Theme,
) -> String {
    if ranges.is_empty() {
        return style_value(value, style, theme);
    }

    let mut rendered = String::new();
    let mut cursor = 0;
    for range in ranges {
        rendered.push_str(&style_value(&value[cursor..range.start], style, theme));
        rendered.push_str(&theme.matched(&value[range.clone()]));
        cursor = range.end;
    }
    rendered.push_str(&style_value(&value[cursor..], style, theme));
    rendered
}

fn intersecting_ranges(
    ranges: &[std::ops::Range<usize>],
    line_start: usize,
    line_end: usize,
) -> Vec<std::ops::Range<usize>> {
    ranges
        .iter()
        .filter_map(|range| {
            let start = range.start.max(line_start);
            let end = range.end.min(line_end);
            (start < end).then_some(start - line_start..end - line_start)
        })
        .collect()
}

const fn access_label(meeting: &Meeting) -> &'static str {
    match meeting.access {
        Access::Online => "Online",
        Access::InPerson => "In person",
        Access::Hybrid => "Hybrid",
        Access::Inactive => "Inactive",
    }
}

fn weekday_name(day: u8) -> &'static str {
    const WEEKDAYS: [&str; 7] = [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];
    WEEKDAYS[usize::from(day)]
}

fn format_clock(time: NaiveTime) -> String {
    let hour = match time.hour() % 12 {
        0 => 12,
        value => value,
    };
    let period = if time.hour() < 12 { "AM" } else { "PM" };
    format!("{hour}:{:02} {period}", time.minute())
}

fn one_line(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use unicode_width::UnicodeWidthStr;

    #[test]
    fn filtering_matches_any_field_and_keeps_the_complete_meeting() {
        let list = fixture_list();

        for query in [
            "harbor",
            "HYBRID",
            "sunday",
            "in progress",
            "brooklyn",
            "zoom.example.test",
            "555-0100",
            "discussion",
            "nyintergroup.org",
        ] {
            assert_eq!(list.matching(query).len(), 1, "query should match: {query}");
        }

        let rendered = list.render_filtered("BROOK", 80, Theme::dark(true));
        assert!(rendered.contains("Harbor Light"));
        assert!(rendered.contains("https://zoom.example.test/harbor"));
        assert!(rendered.contains("https://nyintergroup.org/harbor"));
        assert!(!rendered.contains("Uptown Noon"));
        assert!(rendered.contains("\x1b[1;30;103mBrook\x1b[0m"));
    }

    #[test]
    fn highlighting_marks_every_case_insensitive_match() {
        let highlighted = highlight_matches(
            "Brooklyn brooklyn BROOKLYN",
            "BrOoK",
            FieldStyle::Primary,
            Theme::dark(true),
        );

        assert_eq!(highlighted.matches("\x1b[1;30;103m").count(), 3);
        assert!(highlighted.contains("\x1b[1;30;103mBrook\x1b[0m"));
        assert!(highlighted.contains("\x1b[1;30;103mbrook\x1b[0m"));
        assert!(highlighted.contains("\x1b[1;30;103mBROOK\x1b[0m"));
    }

    #[test]
    fn tables_preserve_terminal_width_for_wide_characters() {
        let table = MeetingTable {
            fields: vec![MeetingField::primary("Meeting", "東 Village Group")],
        };

        for line in table.render(40, "", Theme::dark(false)).lines() {
            assert_eq!(
                UnicodeWidthStr::width(line),
                40,
                "line should occupy the requested terminal width: {line}"
            );
        }
    }

    #[test]
    fn highlighting_survives_a_table_line_wrap() {
        let table = MeetingTable {
            fields: vec![MeetingField::primary("Meeting", "alpha beta")],
        };

        let rendered = table.render(27, "alpha beta", Theme::dark(true));

        assert!(rendered.contains("\x1b[1;30;103malpha\x1b[0m"));
        assert!(rendered.contains("\x1b[1;30;103mbeta\x1b[0m"));
    }

    fn fixture_list() -> MeetingList {
        MeetingList {
            heading: "2 meetings found".to_owned(),
            empty: "No meetings found".to_owned(),
            tables: vec![
                MeetingTable {
                    fields: vec![
                        MeetingField::primary("Meeting", "Harbor Light"),
                        MeetingField::access("Access", "Hybrid", Access::Hybrid),
                        MeetingField::secondary("Schedule", "Sunday 9:00 AM to 10:00 AM"),
                        MeetingField::status("Status", "in progress"),
                        MeetingField::primary("Address", "10 Water St, Brooklyn"),
                        MeetingField::primary("Join", "https://zoom.example.test/harbor"),
                        MeetingField::primary("Phone", "212-555-0100"),
                        MeetingField::secondary("Types", "Open, Discussion"),
                        MeetingField::primary("Source", "https://nyintergroup.org/harbor"),
                    ],
                },
                MeetingTable {
                    fields: vec![
                        MeetingField::primary("Meeting", "Uptown Noon"),
                        MeetingField::access("Access", "Online", Access::Online),
                        MeetingField::secondary("Schedule", "Monday 12:00 PM to 1:00 PM"),
                    ],
                },
            ],
        }
    }
}
