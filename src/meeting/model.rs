use chrono::{NaiveTime, Timelike};
use serde::Deserialize;
use thiserror::Error;

/// A normalized meeting from the New York Inter-Group directory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Meeting {
    pub(super) id: u64,
    pub(super) name: String,
    pub(super) slug: String,
    pub(super) source_url: String,
    pub(super) day: u8,
    pub(super) start: NaiveTime,
    pub(super) end: NaiveTime,
    pub(super) types: Vec<String>,
    pub(super) access: Access,
    pub(super) conference_url: Option<String>,
    pub(super) conference_phone: Option<String>,
    pub(super) conference_url_notes: Option<String>,
    pub(super) conference_phone_notes: Option<String>,
    pub(super) location: Option<String>,
    pub(super) address: Option<String>,
    pub(super) regions: Vec<String>,
    pub(super) group: Option<String>,
}

impl Meeting {
    /// Returns the directory identifier.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Returns the meeting name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the canonical NYIG meeting page.
    #[must_use]
    pub fn source_url(&self) -> &str {
        &self.source_url
    }

    /// Returns the online join URL when one is listed.
    #[must_use]
    pub fn join_url(&self) -> Option<&str> {
        self.conference_url.as_deref()
    }

    /// Returns whether the meeting offers online access.
    #[must_use]
    pub const fn is_online(&self) -> bool {
        matches!(self.access, Access::Online | Access::Hybrid)
    }

    /// Returns whether the meeting offers in-person access.
    #[must_use]
    pub const fn is_in_person(&self) -> bool {
        matches!(self.access, Access::InPerson | Access::Hybrid)
    }

    pub(super) fn is_active(&self) -> bool {
        self.access != Access::Inactive
    }

    pub(super) fn start_minutes(&self) -> i64 {
        i64::from(self.start.num_seconds_from_midnight() / 60)
    }

    pub(super) fn end_minutes(&self) -> i64 {
        i64::from(self.end.num_seconds_from_midnight() / 60)
    }

    pub(super) fn search_text(&self) -> String {
        let mut values = vec![self.name.as_str(), self.slug.as_str()];
        values.extend(self.group.as_deref());
        values.extend(self.location.as_deref());
        values.extend(self.address.as_deref());
        values.extend(self.regions.iter().map(String::as_str));
        values.join(" ").to_lowercase()
    }

    pub(super) fn region_text(&self) -> String {
        self.regions.join(" ").to_lowercase()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(super) enum Access {
    Online,
    InPerson,
    Hybrid,
    #[serde(other)]
    Inactive,
}

#[derive(Deserialize)]
struct RawMeeting {
    id: u64,
    name: String,
    slug: String,
    url: String,
    day: u8,
    time: String,
    end_time: Option<String>,
    #[serde(default)]
    types: Vec<String>,
    attendance_option: Access,
    conference_url: Option<String>,
    conference_phone: Option<String>,
    conference_url_notes: Option<String>,
    conference_phone_notes: Option<String>,
    location: Option<String>,
    formatted_address: Option<String>,
    #[serde(default)]
    regions: Vec<String>,
    group: Option<String>,
}

/// A failure while retrieving or decoding the NYIG directory feed.
#[derive(Debug, Error)]
pub enum DirectoryError {
    /// A directory page or data request failed.
    #[error("could not fetch the NYIG meeting directory: {0}")]
    Network(#[from] ureq::Error),
    /// The configured directory page URL was invalid.
    #[error("invalid NYIG directory URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    /// The directory page did not advertise its structured data feed.
    #[error("NYIG meeting page does not advertise a directory data source")]
    MissingDataSource,
    /// The feed was not valid JSON or did not match the expected schema.
    #[error("could not decode the NYIG meeting directory: {0}")]
    InvalidJson(#[from] serde_json::Error),
    /// A record contained a weekday outside Sunday through Saturday.
    #[error("NYIG meeting '{name}' has invalid weekday {day}")]
    InvalidDay { name: String, day: u8 },
    /// A record contained an invalid 24-hour time.
    #[error("NYIG meeting '{name}' has invalid time '{time}'")]
    InvalidTime { name: String, time: String },
}

/// Parses the complete structured NYIG meeting feed.
pub fn parse_directory(input: &str) -> Result<Vec<Meeting>, DirectoryError> {
    serde_json::from_str::<Vec<RawMeeting>>(input)?
        .into_iter()
        .map(Meeting::try_from)
        .collect()
}

impl TryFrom<RawMeeting> for Meeting {
    type Error = DirectoryError;

    fn try_from(raw: RawMeeting) -> Result<Self, Self::Error> {
        let name = decode_entities(raw.name);
        if raw.day > 6 {
            return Err(DirectoryError::InvalidDay { name, day: raw.day });
        }
        let start = parse_time(&name, &raw.time)?;
        let end = match raw.end_time {
            Some(end) => parse_time(&name, &end)?,
            None => start + chrono::Duration::minutes(60),
        };

        Ok(Self {
            id: raw.id,
            name,
            slug: raw.slug,
            source_url: decode_entities(raw.url),
            day: raw.day,
            start,
            end,
            types: raw.types.into_iter().map(decode_entities).collect(),
            access: raw.attendance_option,
            conference_url: raw.conference_url.map(decode_entities),
            conference_phone: raw.conference_phone.map(decode_entities),
            conference_url_notes: raw.conference_url_notes.map(decode_entities),
            conference_phone_notes: raw.conference_phone_notes.map(decode_entities),
            location: raw.location.map(decode_entities),
            address: raw.formatted_address.map(decode_entities),
            regions: raw.regions.into_iter().map(decode_entities).collect(),
            group: raw.group.map(decode_entities),
        })
    }
}

fn parse_time(name: &str, value: &str) -> Result<NaiveTime, DirectoryError> {
    NaiveTime::parse_from_str(value, "%H:%M").map_err(|_| DirectoryError::InvalidTime {
        name: name.to_owned(),
        time: value.to_owned(),
    })
}

fn decode_entities(value: String) -> String {
    if value.contains('&') {
        html_escape::decode_html_entities(&value).into_owned()
    } else {
        value
    }
}
