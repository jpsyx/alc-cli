use std::{
    env, fs, io,
    path::{Path, PathBuf},
    process,
    time::Duration,
};

use chrono::NaiveDate;
use scraper::{Html, Selector};
use url::Url;

use super::model::{DirectoryError, Meeting, parse_directory};

const DIRECTORY_PAGE_URL: &str = "https://www.nyintergroup.org/meetings/";
const CACHE_FORMAT: &str = "alc-meetings-v1";
const CACHE_FILE_NAME: &str = "meetings.cache";

/// Resolves the structured feed advertised by the NYIG meeting page.
pub fn directory_data_url(page_url: &str, page: &str) -> Result<String, DirectoryError> {
    let document = Html::parse_document(page);
    let selector = Selector::parse("#tsml-ui[data-src]")
        .expect("the fixed meeting data selector should be valid");
    let source = document
        .select(&selector)
        .next()
        .and_then(|element| element.value().attr("data-src"))
        .ok_or(DirectoryError::MissingDataSource)?;

    Ok(Url::parse(page_url)?.join(source)?.to_string())
}

/// Fetches meetings from New York Inter-Group's advertised structured feed.
pub fn fetch() -> Result<Vec<Meeting>, DirectoryError> {
    fetch_from(DIRECTORY_PAGE_URL)
}

/// Fetches meetings from a compatible page and its advertised data source.
pub fn fetch_from(page_url: &str) -> Result<Vec<Meeting>, DirectoryError> {
    parse_directory(&fetch_directory(page_url)?)
}

/// Fetches the NYIG directory unless today's validated cache can be reused.
pub fn fetch_cached(today: NaiveDate) -> Result<Vec<Meeting>, DirectoryError> {
    fetch_cached_page(DIRECTORY_PAGE_URL, today)
}

/// Fetches a compatible directory page unless today's validated cache can be reused.
pub fn fetch_cached_page(page_url: &str, today: NaiveDate) -> Result<Vec<Meeting>, DirectoryError> {
    default_cache_path().map_or_else(
        || fetch_from(page_url),
        |cache_path| fetch_cached_from(page_url, &cache_path, today),
    )
}

/// Uses a validated daily cache at an explicit path, fetching on a cache miss.
pub fn fetch_cached_from(
    page_url: &str,
    cache_path: &Path,
    today: NaiveDate,
) -> Result<Vec<Meeting>, DirectoryError> {
    if let Some(meetings) = read_cache(cache_path, page_url, today) {
        return Ok(meetings);
    }

    let directory = fetch_directory(page_url)?;
    let meetings = parse_directory(&directory)?;
    let _cache_result = write_cache(cache_path, page_url, today, &directory);

    Ok(meetings)
}

/// Returns the platform-standard path for alc's meeting cache.
#[must_use]
pub fn default_cache_path() -> Option<PathBuf> {
    env::var_os("ALC_CACHE_DIR")
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .or_else(platform_cache_directory)
        .map(|directory| directory.join("alc").join(CACHE_FILE_NAME))
}

fn fetch_directory(page_url: &str) -> Result<String, DirectoryError> {
    let config = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(15)))
        .user_agent(format!("alc/{}", env!("CARGO_PKG_VERSION")))
        .build();
    let agent: ureq::Agent = config.into();
    let page = agent.get(page_url).call()?.body_mut().read_to_string()?;
    let data_url = directory_data_url(page_url, &page)?;
    let directory = agent.get(data_url).call()?.body_mut().read_to_string()?;

    Ok(directory)
}

fn read_cache(cache_path: &Path, page_url: &str, today: NaiveDate) -> Option<Vec<Meeting>> {
    let cache = fs::read_to_string(cache_path).ok()?;
    let mut sections = cache.splitn(4, '\n');
    let format = sections.next()?;
    let cached_date = sections.next()?;
    let cached_page_url = sections.next()?;
    let directory = sections.next()?;

    if format != CACHE_FORMAT || cached_date != today.to_string() || cached_page_url != page_url {
        return None;
    }

    parse_directory(directory).ok()
}

fn write_cache(
    cache_path: &Path,
    page_url: &str,
    today: NaiveDate,
    directory: &str,
) -> io::Result<()> {
    let parent = cache_path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "meeting cache path has no parent directory",
        )
    })?;
    fs::create_dir_all(parent)?;

    let cache = format!("{CACHE_FORMAT}\n{today}\n{page_url}\n{directory}");
    let temporary_path = cache_path.with_extension(format!("tmp-{}", process::id()));
    fs::write(&temporary_path, cache.as_bytes())?;

    if let Err(rename_error) = fs::rename(&temporary_path, cache_path) {
        let direct_write = fs::write(cache_path, cache.as_bytes());
        let _remove_result = fs::remove_file(&temporary_path);
        direct_write.map_err(|_| rename_error)?;
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn platform_cache_directory() -> Option<PathBuf> {
    env::var_os("HOME").map(|home| PathBuf::from(home).join("Library").join("Caches"))
}

#[cfg(target_os = "windows")]
fn platform_cache_directory() -> Option<PathBuf> {
    env::var_os("LOCALAPPDATA").map(PathBuf::from)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn platform_cache_directory() -> Option<PathBuf> {
    env::var_os("XDG_CACHE_HOME")
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
}
