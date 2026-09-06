use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    process, thread,
    time::{SystemTime, UNIX_EPOCH},
};

use alc::meeting;
use chrono::NaiveDate;

const DIRECTORY: &str = r#"[{
  "id": 42,
  "name": "Source Boundary",
  "slug": "source-boundary",
  "url": "https://www.nyintergroup.org/meetings/source-boundary/",
  "day": 1,
  "time": "08:00",
  "end_time": "09:00",
  "attendance_option": "online"
}]"#;

const UPDATED_DIRECTORY: &str = r#"[{
  "id": 43,
  "name": "Fresh Directory",
  "slug": "fresh-directory",
  "url": "https://www.nyintergroup.org/meetings/fresh-directory/",
  "day": 2,
  "time": "09:00",
  "end_time": "10:00",
  "attendance_option": "in_person"
}]"#;

#[test]
fn resolves_the_cache_url_advertised_by_the_meeting_page() {
    let page = r#"<html><div id="tsml-ui" data-src="/wp-content/meetings.json?42"></div></html>"#;

    let url = meeting::directory_data_url("https://www.nyintergroup.org/meetings/", page)
        .expect("cache URL should resolve");

    assert_eq!(
        url,
        "https://www.nyintergroup.org/wp-content/meetings.json?42"
    );
}

#[test]
fn fetches_the_page_then_its_advertised_directory_feed() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("test server should bind");
    let address = listener
        .local_addr()
        .expect("test server should have an address");
    let server = thread::spawn(move || {
        respond_once(
            &listener,
            "/meetings/",
            "text/html",
            r#"<div id="tsml-ui" data-src="/cache.json?revision=7"></div>"#,
        );
        respond_once(
            &listener,
            "/cache.json?revision=7",
            "application/json",
            DIRECTORY,
        );
    });

    let meetings = meeting::fetch_from(&format!("http://{address}/meetings/"))
        .expect("directory should be fetched");
    server.join().expect("test server should finish");

    assert_eq!(meetings.len(), 1);
    assert_eq!(meetings[0].name(), "Source Boundary");
}

#[test]
fn reuses_a_valid_cache_for_the_same_new_york_date() {
    let temporary = TestDirectory::new();
    let cache_path = temporary.path().join("meetings.cache");
    let date = NaiveDate::from_ymd_opt(2026, 9, 6).expect("fixture date should be valid");
    let (page_url, server) = serve_directory(DIRECTORY);

    let first = meeting::fetch_cached_from(&page_url, &cache_path, date)
        .expect("first request should fetch and cache");
    server.join().expect("test server should finish");
    let second = meeting::fetch_cached_from(&page_url, &cache_path, date)
        .expect("second request should use the cache after the server stops");

    assert_eq!(first[0].name(), "Source Boundary");
    assert_eq!(second[0].name(), "Source Boundary");
    assert!(cache_path.is_file());
}

#[test]
fn refreshes_the_cache_when_the_new_york_date_changes() {
    let temporary = TestDirectory::new();
    let cache_path = temporary.path().join("meetings.cache");
    let first_date = NaiveDate::from_ymd_opt(2026, 9, 6).expect("fixture date should be valid");
    let next_date = NaiveDate::from_ymd_opt(2026, 9, 7).expect("fixture date should be valid");
    let (first_url, first_server) = serve_directory(DIRECTORY);
    meeting::fetch_cached_from(&first_url, &cache_path, first_date)
        .expect("first day should be cached");
    first_server.join().expect("first server should finish");
    let (next_url, next_server) = serve_directory(UPDATED_DIRECTORY);

    let refreshed = meeting::fetch_cached_from(&next_url, &cache_path, next_date)
        .expect("next day should refresh");
    next_server.join().expect("next server should finish");
    let reused = meeting::fetch_cached_from(&next_url, &cache_path, next_date)
        .expect("refreshed data should be reusable without the server");

    assert_eq!(refreshed[0].name(), "Fresh Directory");
    assert_eq!(reused[0].name(), "Fresh Directory");
}

#[test]
fn replaces_a_malformed_cache_instead_of_treating_it_as_directory_data() {
    let temporary = TestDirectory::new();
    let cache_path = temporary.path().join("meetings.cache");
    fs::write(&cache_path, "alc-meetings-v1\n2026-09-06\ninvalid")
        .expect("malformed cache fixture should be written");
    let date = NaiveDate::from_ymd_opt(2026, 9, 6).expect("fixture date should be valid");
    let (page_url, server) = serve_directory(DIRECTORY);

    let meetings = meeting::fetch_cached_from(&page_url, &cache_path, date)
        .expect("a malformed cache should be refreshed");
    server.join().expect("test server should finish");

    assert_eq!(meetings[0].name(), "Source Boundary");
}

#[test]
fn an_unwritable_cache_does_not_block_a_successful_directory_fetch() {
    let temporary = TestDirectory::new();
    let blocked_directory = temporary.path().join("not-a-directory");
    fs::write(&blocked_directory, "fixture").expect("blocking file should be written");
    let cache_path = blocked_directory.join("meetings.cache");
    let date = NaiveDate::from_ymd_opt(2026, 9, 6).expect("fixture date should be valid");
    let (page_url, server) = serve_directory(DIRECTORY);

    let meetings = meeting::fetch_cached_from(&page_url, &cache_path, date)
        .expect("cache write failure should not hide fetched meetings");
    server.join().expect("test server should finish");

    assert_eq!(meetings[0].name(), "Source Boundary");
}

#[cfg(target_os = "macos")]
#[test]
fn default_cache_uses_the_macos_cache_directory() {
    let cache_path = meeting::default_cache_path().expect("home directory should be available");

    assert!(cache_path.ends_with("Library/Caches/alc/meetings.cache"));
}

fn serve_directory(directory: &'static str) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("test server should bind");
    let address = listener
        .local_addr()
        .expect("test server should have an address");
    let server = thread::spawn(move || {
        respond_once(
            &listener,
            "/meetings/",
            "text/html",
            r#"<div id="tsml-ui" data-src="/cache.json"></div>"#,
        );
        respond_once(&listener, "/cache.json", "application/json", directory);
    });
    (format!("http://{address}/meetings/"), server)
}

fn respond_once(listener: &TcpListener, path: &str, content_type: &str, body: &str) {
    let (mut stream, _) = listener.accept().expect("test server should accept");
    let mut request = [0_u8; 2_048];
    let bytes_read = stream
        .read(&mut request)
        .expect("request should be readable");
    let request = String::from_utf8_lossy(&request[..bytes_read]);
    assert!(request.starts_with(&format!("GET {path} HTTP/1.1\r\n")));

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    stream
        .write_all(response.as_bytes())
        .expect("response should be writable");
}

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should follow the Unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("alc-cache-{}-{unique}", process::id()));
        fs::create_dir(&path).expect("temporary directory should be created");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("temporary directory should be removable");
    }
}
