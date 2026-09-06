//! New York Inter-Group meeting source, model, queries, and rendering.

use std::io;

use chrono::DateTime;
use chrono_tz::Tz;

use crate::theme::Theme;

mod model;
mod pager;
mod query;
mod render;
mod source;
mod table;

pub use model::{DirectoryError, Meeting, parse_directory};
pub use query::{Availability, AvailableMeeting, find, now, schedule};
pub use render::{directions_url, render_find, render_find_at, render_now};
pub use source::{
    default_cache_path, directory_data_url, fetch, fetch_cached, fetch_cached_from,
    fetch_cached_page, fetch_from,
};

/// Displays meeting search results in a pager when appropriate, or prints them directly.
pub fn display_find(
    meetings: &[&Meeting],
    origin: Option<&str>,
    at: &DateTime<Tz>,
    pager_enabled: bool,
    theme: Theme,
) -> io::Result<()> {
    pager::display(
        &render::find_list_at(meetings, origin, at),
        pager_enabled,
        theme,
    )
}

/// Displays current meeting results in a pager when appropriate, or prints them directly.
pub fn display_now(
    meetings: &[AvailableMeeting<'_>],
    origin: Option<&str>,
    pager_enabled: bool,
    theme: Theme,
) -> io::Result<()> {
    pager::display(&render::now_list(meetings, origin), pager_enabled, theme)
}
