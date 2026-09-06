use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_nyintergroup"))
        .args(args)
        .output()
        .expect("nyintergroup should run")
}

#[test]
fn version_reports_the_command_and_crate_version() {
    let output = run(&["--version"]);
    let expected = format!("nyintergroup {}\n", env!("CARGO_PKG_VERSION"));

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("version should be UTF-8"),
        expected
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn a_bare_invocation_prints_help() {
    let output = run(&[]);
    let stdout = String::from_utf8(output.stdout).expect("help should be UTF-8");

    assert!(output.status.success());
    assert!(stdout.contains("Find New York Inter-Group AA meetings from the terminal"));
    assert!(stdout.contains("Usage: nyintergroup"));
    assert!(output.stderr.is_empty());
}
