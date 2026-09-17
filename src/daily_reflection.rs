//! Daily Reflection dates and retrieval.

use std::{fmt, time::Duration};

use chrono::NaiveDate;
use scraper::{ElementRef, Html, Selector};
use serde::Deserialize;
use thiserror::Error;

use crate::{text_wrap::wrap_prose, theme::Theme};

const SOURCE_BASE_URL: &str = "https://www.aa.org/daily-reflections";
const API_BASE_URL: &str = "https://www.aa.org/api/reflections";

/// A validated calendar date for a Daily Reflection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReflectionDate(NaiveDate);

impl ReflectionDate {
    /// Parses either `YYYY-MM-DD` or `MM-DD`.
    ///
    /// `current_year` supplies the omitted year in the short form.
    pub fn parse(input: &str, current_year: i32) -> Result<Self, ParseReflectionDateError> {
        let parts = input.split('-').collect::<Vec<_>>();
        let parsed = match parts.as_slice() {
            [year, month, day] if year.len() == 4 && month.len() == 2 && day.len() == 2 => {
                parse_date(year, month, day)
            }
            [month, day] if month.len() == 2 && day.len() == 2 => {
                parse_date(&current_year.to_string(), month, day)
            }
            _ => None,
        };

        parsed
            .map(Self)
            .ok_or_else(|| ParseReflectionDateError(input.to_owned()))
    }

    /// Returns the month and day path consumed by AA.org's reflection API.
    #[must_use]
    pub fn api_path(self) -> String {
        self.0.format("%m/%d").to_string()
    }

    fn api_date(self) -> String {
        self.0.format("%m-%d").to_string()
    }

    fn source_url(self) -> String {
        format!("{SOURCE_BASE_URL}?date=00-{}", self.api_date())
    }
}

impl fmt::Display for ReflectionDate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.format("%Y-%m-%d").fmt(formatter)
    }
}

impl From<NaiveDate> for ReflectionDate {
    fn from(date: NaiveDate) -> Self {
        Self(date)
    }
}

/// An invalid Daily Reflection date supplied by the user.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseReflectionDateError(String);

impl fmt::Display for ParseReflectionDateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid date '{}'; expected YYYY-MM-DD or MM-DD",
            self.0
        )
    }
}

impl std::error::Error for ParseReflectionDateError {}

/// A Daily Reflection converted from AA.org's response into plain text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DailyReflection {
    title: String,
    date_label: String,
    paragraphs: Vec<String>,
    copyright: String,
    source_url: String,
}

impl DailyReflection {
    /// Returns the reflection title.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the human-readable month and day supplied by AA.org.
    #[must_use]
    pub fn date_label(&self) -> &str {
        &self.date_label
    }

    /// Returns the reflection's ordered plain-text paragraphs.
    #[must_use]
    pub fn paragraphs(&self) -> &[String] {
        &self.paragraphs
    }

    /// Returns AA World Services' copyright attribution.
    #[must_use]
    pub fn copyright(&self) -> &str {
        &self.copyright
    }

    /// Returns the official page for the selected reflection date.
    #[must_use]
    pub fn source_url(&self) -> &str {
        &self.source_url
    }
}

/// A failure while decoding AA.org's Daily Reflection response.
#[derive(Debug, Error)]
pub enum DailyReflectionError {
    /// The reflection request or response body failed.
    #[error("could not fetch AA.org reflection: {0}")]
    Network(#[from] ureq::Error),
    /// The endpoint did not return valid JSON.
    #[error("could not decode AA.org reflection response: {0}")]
    InvalidJson(#[from] serde_json::Error),
    /// The endpoint reported that no reflection was available.
    #[error("AA.org returned reflection status {0}")]
    Upstream(u16),
    /// Required markup was absent from the endpoint's HTML fragment.
    #[error("AA.org reflection response is missing {0}")]
    MissingField(&'static str),
    /// The response did not match the requested month and day.
    #[error("AA.org returned reflection {actual} when {expected} was requested")]
    WrongDate { expected: String, actual: String },
}

#[derive(Deserialize)]
struct ApiResponse {
    err: u16,
    data: String,
}

/// Parses the structured response from AA.org's `/api/reflections/MM/DD` route.
pub fn parse_api_response(
    response: &str,
    expected_date: ReflectionDate,
) -> Result<DailyReflection, DailyReflectionError> {
    let response: ApiResponse = serde_json::from_str(response)?;
    if response.err != 200 {
        return Err(DailyReflectionError::Upstream(response.err));
    }

    let document = Html::parse_fragment(&response.data);
    let article_selector = selector("article[data-date]");
    let article = document
        .select(&article_selector)
        .next()
        .ok_or(DailyReflectionError::MissingField("reflection article"))?;
    let actual_date = article
        .value()
        .attr("data-date")
        .ok_or(DailyReflectionError::MissingField("reflection date"))?;
    let expected_api_date = expected_date.api_date();
    if actual_date != expected_api_date {
        return Err(DailyReflectionError::WrongDate {
            expected: expected_api_date,
            actual: actual_date.to_owned(),
        });
    }

    let title = select_text(&article, ".field--name-title", "title")?;
    let date_label = select_text(&article, ".field--name-field-date", "date label")?;
    let copyright = select_text(
        &article,
        ".field--name-field-copyright .field--name-description",
        "copyright",
    )?;
    let paragraph_selector = selector(".field--name-body p");
    let paragraphs = article
        .select(&paragraph_selector)
        .map(element_text)
        .filter(|paragraph| !paragraph.is_empty())
        .collect::<Vec<_>>();
    if paragraphs.is_empty() {
        return Err(DailyReflectionError::MissingField("body"));
    }

    Ok(DailyReflection {
        title,
        date_label,
        paragraphs,
        copyright,
        source_url: expected_date.source_url(),
    })
}

/// Fetches a Daily Reflection from AA.org's official endpoint.
pub fn fetch(date: ReflectionDate) -> Result<DailyReflection, DailyReflectionError> {
    fetch_from(API_BASE_URL, date)
}

/// Fetches a Daily Reflection from a compatible endpoint.
///
/// The configurable base keeps the HTTP boundary independently testable while
/// [`fetch`] remains the production entry point.
pub fn fetch_from(
    base_url: &str,
    date: ReflectionDate,
) -> Result<DailyReflection, DailyReflectionError> {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(15)))
        .user_agent(format!("alc/{}", env!("CARGO_PKG_VERSION")))
        .accept("application/json")
        .build();
    let agent: ureq::Agent = config.into();
    let url = format!("{}/{}", base_url.trim_end_matches('/'), date.api_path());
    let response = agent.get(url).call()?.body_mut().read_to_string()?;

    parse_api_response(&response, date)
}

/// Renders a Daily Reflection for human-readable terminal output.
///
/// Prose is soft-wrapped for comfortable reading. The source line is left
/// intact so the URL stays selectable and clickable.
#[must_use]
pub fn render(reflection: &DailyReflection, theme: Theme) -> String {
    let heading = theme.heading(&format!("Daily Reflection | {}", reflection.date_label));
    let title = wrapped(&reflection.title, |line| theme.value(line));
    let body = reflection
        .paragraphs
        .iter()
        .map(|paragraph| wrap_prose(paragraph).join("\n"))
        .collect::<Vec<_>>()
        .join("\n\n");
    let source = format!(
        "{} {}",
        theme.accent("Source:"),
        theme.value(&reflection.source_url)
    );
    let copyright = wrapped(&reflection.copyright, |line| theme.muted(line));
    format!("{heading}\n\n{title}\n\n{body}\n\n{source}\n{copyright}\n")
}

/// Wraps text and styles each resulting line, so no escape sequence spans a
/// line break.
fn wrapped(text: &str, style: impl Fn(&str) -> String) -> String {
    wrap_prose(text)
        .iter()
        .map(|line| style(line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn parse_date(year: &str, month: &str, day: &str) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(year.parse().ok()?, month.parse().ok()?, day.parse().ok()?)
}

fn selector(value: &str) -> Selector {
    Selector::parse(value).expect("internal CSS selectors should be valid")
}

fn select_text(
    article: &ElementRef<'_>,
    selector_value: &str,
    field_name: &'static str,
) -> Result<String, DailyReflectionError> {
    let value = article
        .select(&selector(selector_value))
        .next()
        .map(element_text)
        .filter(|text| !text.is_empty())
        .ok_or(DailyReflectionError::MissingField(field_name))?;
    Ok(value)
}

fn element_text(element: ElementRef<'_>) -> String {
    element
        .text()
        .flat_map(str::split_whitespace)
        .collect::<Vec<_>>()
        .join(" ")
}
