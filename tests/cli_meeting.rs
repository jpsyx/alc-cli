use std::{
    fs,
    io::{ErrorKind, Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    process::{self, Command},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const DIRECTORY: &str = r#"[{
  "id": 42,
  "name": "CLI Boundary Group",
  "slug": "cli-boundary-group",
  "url": "https://www.nyintergroup.org/meetings/cli-boundary-group/",
  "day": 1,
  "time": "08:00",
  "end_time": "09:00",
  "types": ["O", "ONL"],
  "conference_url": "https://meet.example.test/cli",
  "formatted_address": "New York, NY, USA",
  "regions": ["Manhattan"],
  "attendance_option": "online"
}]"#;

#[test]
fn meeting_find_fetches_then_reuses_the_daily_cache() {
    let temporary = TestDirectory::new();
    let listener = TcpListener::bind("127.0.0.1:0").expect("test server should bind");
    let address = listener
        .local_addr()
        .expect("test server should have an address");
    let server = thread::spawn(move || {
        respond_once(
            &listener,
            "/meetings/",
            "text/html",
            r#"<div id="tsml-ui" data-src="/directory.json"></div>"#,
        );
        respond_once(&listener, "/directory.json", "application/json", DIRECTORY);
    });

    let output = Command::new(env!("CARGO_BIN_EXE_alc"))
        .args(["meeting", "find", "cli boundary"])
        .env("ALC_NYIG_URL", format!("http://{address}/meetings/"))
        .env("ALC_CACHE_DIR", temporary.path())
        .output()
        .expect("alc should run");
    server.join().expect("test server should finish");
    let stdout = String::from_utf8(output.stdout).expect("output should be UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("diagnostics should be UTF-8");

    assert!(output.status.success());
    assert!(stdout.contains("1 meeting found"));
    assert!(stdout.contains("│ Meeting        │ CLI Boundary Group"));
    assert!(stdout.contains("│ Join           │ https://meet.example.test/cli"));
    assert!(
        stdout.contains(
            "│ Source         │ https://www.nyintergroup.org/meetings/cli-boundary-group/"
        )
    );
    assert_eq!(stderr, "● Loading New York Inter-Group meetings...\n");

    let cached_output = Command::new(env!("CARGO_BIN_EXE_alc"))
        .args(["meeting", "find", "cli boundary"])
        .env("ALC_NYIG_URL", format!("http://{address}/meetings/"))
        .env("ALC_CACHE_DIR", temporary.path())
        .output()
        .expect("alc should run from its cache");
    let cached_stdout =
        String::from_utf8(cached_output.stdout).expect("cached output should be UTF-8");

    assert!(cached_output.status.success());
    assert!(cached_stdout.contains("CLI Boundary Group"));
}

#[test]
fn meeting_source_failures_use_a_clear_error_diagnostic() {
    let temporary = TestDirectory::new();

    let output = Command::new(env!("CARGO_BIN_EXE_alc"))
        .args(["meeting", "find"])
        .env("ALC_NYIG_URL", "not-a-valid-url")
        .env("ALC_CACHE_DIR", temporary.path())
        .output()
        .expect("alc should run");
    let stderr = String::from_utf8(output.stderr).expect("diagnostics should be UTF-8");

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(stderr.starts_with("● Loading New York Inter-Group meetings...\n✗ "));
    assert!(!stderr.contains("\nerror:"));
}

fn respond_once(listener: &TcpListener, path: &str, content_type: &str, body: &str) {
    listener
        .set_nonblocking(true)
        .expect("test server should become nonblocking");
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut stream = loop {
        match listener.accept() {
            Ok((stream, _)) => break stream,
            Err(error) if error.kind() == ErrorKind::WouldBlock && Instant::now() < deadline => {
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("test server should accept: {error}"),
        }
    };
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
        let path = std::env::temp_dir().join(format!("alc-cli-cache-{}-{unique}", process::id()));
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
