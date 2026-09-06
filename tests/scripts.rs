use std::{
    fs,
    path::{Path, PathBuf},
    process::{self, Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

fn help(script: &str) -> Output {
    Command::new("/bin/bash")
        .args([script, "--help"])
        .env("PATH", "")
        .output()
        .expect("script should run")
}

#[test]
fn run_launcher_prints_help_before_tool_or_build_checks() {
    let output = help("run.sh");
    let stdout = String::from_utf8(output.stdout).expect("help should be UTF-8");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert!(stdout.contains("Find AA meetings and read Daily Reflections"));
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("./run.sh meeting"));
    assert!(stdout.contains("./run.sh daily"));
}

#[test]
fn installer_prints_help_before_tool_checks_and_includes_examples() {
    let output = help("install.sh");
    let stdout = String::from_utf8(output.stdout).expect("help should be UTF-8");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert!(stdout.contains("Install alc for use from any directory"));
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("./install.sh"));
    assert!(stdout.contains("BIN_DIR=\"$HOME/bin\" ./install.sh"));
}

#[test]
fn installer_reports_build_and_success_with_clear_status_markers() {
    let temporary = TestDirectory::new();
    let output = Command::new("/bin/bash")
        .arg("install.sh")
        .env("BIN_DIR", temporary.path())
        .output()
        .expect("installer should run");
    let stdout = String::from_utf8(output.stdout).expect("output should be UTF-8");
    let stderr = String::from_utf8(output.stderr).expect("diagnostics should be UTF-8");

    assert!(output.status.success());
    assert!(temporary.path().join("alc").is_file());
    assert!(stdout.starts_with("✓ Installed alc\n"));
    assert!(stdout.contains(&temporary.path().join("alc").display().to_string()));
    assert!(stderr.contains("● Building alc (release)..."));
    assert!(stderr.contains("! The installation directory is not on PATH."));
}

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should follow the Unix epoch")
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("alc-install-{}-{unique}", process::id()));
        fs::create_dir(&directory).expect("temporary directory should be created");
        Self(directory)
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
