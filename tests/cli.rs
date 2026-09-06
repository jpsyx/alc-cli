use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_alc"))
        .args(args)
        .output()
        .expect("alc should run")
}

#[test]
fn version_reports_the_command_and_crate_version() {
    let output = run(&["--version"]);
    let expected = format!("alc {}\n", env!("CARGO_PKG_VERSION"));

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
    let explicit_help = run(&["--help"]);

    assert!(output.status.success());
    assert!(stdout.contains("Find AA meetings and read Daily Reflections from the terminal"));
    assert!(stdout.contains("Usage: alc <COMMAND>"));
    assert!(stdout.contains("meeting"));
    assert!(stdout.contains("daily-reflection"));
    assert_eq!(stdout.as_bytes(), explicit_help.stdout);
    assert!(output.stderr.is_empty());
}

#[test]
fn invalid_reflection_date_exits_before_fetching() {
    let output = run(&["daily", "--date", "2026-02-30"]);
    let stderr = String::from_utf8(output.stderr).expect("error should be UTF-8");

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(stderr.contains("invalid date '2026-02-30'; expected YYYY-MM-DD or MM-DD"));
}

#[test]
fn meeting_help_lists_commands_and_schedule_shortcuts() {
    let output = run(&["meeting", "--help"]);
    let stdout = String::from_utf8(output.stdout).expect("help should be UTF-8");

    assert!(output.status.success());
    assert!(stdout.contains("Usage: alc meeting [COMMAND]"));
    assert!(stdout.contains("now"));
    assert!(stdout.contains("find"));
    for shortcut in [
        "today",
        "tomorrow",
        "week",
        "sunday",
        "tuesday",
        "saturday",
        "morning",
        "afternoon",
        "night",
        "evening",
        "this week",
    ] {
        assert!(stdout.contains(shortcut), "missing {shortcut:?} from help");
    }
    assert!(output.stderr.is_empty());
}

#[test]
fn every_schedule_shortcut_has_help_and_examples() {
    for shortcut in [
        ["meeting", "today", "--help"].as_slice(),
        ["meeting", "tomorrow", "--help"].as_slice(),
        ["meeting", "week", "--help"].as_slice(),
        ["meeting", "tuesday", "--help"].as_slice(),
        ["meeting", "morning", "--help"].as_slice(),
        ["meeting", "afternoon", "--help"].as_slice(),
        ["meeting", "night", "--help"].as_slice(),
        ["meeting", "evening", "--help"].as_slice(),
        ["meeting", "this", "week", "--help"].as_slice(),
    ] {
        let output = run(shortcut);
        let stdout = String::from_utf8(output.stdout).expect("help should be UTF-8");

        assert!(
            output.status.success(),
            "help should succeed for {shortcut:?}"
        );
        assert!(stdout.contains("Examples:"));
        assert!(stdout.contains("alc meeting today --type online"));
        assert!(stdout.contains("alc meeting this week morning"));
        assert!(stdout.contains("j/k or arrow keys"));
        assert!(stdout.contains("a shows all"));
        assert!(stdout.contains("r resets the original view"));
    }
}

#[test]
fn meeting_list_help_documents_pager_controls_and_plain_output() {
    for command in [["meeting", "now", "--help"], ["meeting", "find", "--help"]] {
        let output = run(&command);
        let stdout = String::from_utf8(output.stdout).expect("help should be UTF-8");

        assert!(output.status.success());
        assert!(stdout.contains("--no-pager"));
        assert!(stdout.contains("j/k or arrow keys"));
        assert!(stdout.contains("d/u"));
        assert!(stdout.contains("/ filters"));
        assert!(stdout.contains("G jumps to the end"));
        assert!(stdout.contains("q quits"));
        assert!(stdout.contains("a shows all"));
        assert!(stdout.contains("h selects hybrid"));
        assert!(stdout.contains("p selects in-person"));
        assert!(stdout.contains("o selects online"));
        assert!(stdout.contains("r resets the original view"));
        assert!(stdout.contains("Ctrl+U clears the filter"));
        assert!(stdout.contains("Backspace on an empty filter exits editing"));
        assert!(stdout.contains("In-progress meetings show how long ago they started"));
    }
}

#[test]
fn meeting_now_help_documents_the_in_progress_cutoff() {
    let output = run(&["meeting", "now", "--help"]);
    let stdout = String::from_utf8(output.stdout).expect("help should be UTF-8");

    assert!(output.status.success());
    assert!(stdout.contains("started within the last 30 minutes"));
}

#[test]
fn every_command_help_includes_examples_and_implicit_defaults() {
    let cases = [
        (
            ["--help"].as_slice(),
            ["alc meeting", "alc daily"].as_slice(),
            None,
        ),
        (
            ["meeting", "--help"].as_slice(),
            ["alc meeting now", "alc meeting find"].as_slice(),
            Some("With no command, `alc meeting` runs `alc meeting now`."),
        ),
        (
            ["meeting", "now", "--help"].as_slice(),
            ["alc meeting now", "--type online"].as_slice(),
            Some("By default, all active access types are included."),
        ),
        (
            ["meeting", "find", "--help"].as_slice(),
            ["alc meeting find", "--weekday sunday"].as_slice(),
            Some("With no filters, searches all active meetings."),
        ),
        (
            ["daily", "--help"].as_slice(),
            ["alc daily", "--date 09-06"].as_slice(),
            Some("Defaults to today's local date."),
        ),
    ];

    for (args, examples, default_explanation) in cases {
        let output = run(args);
        let stdout = String::from_utf8(output.stdout).expect("help should be UTF-8");

        assert!(output.status.success(), "help should succeed for {args:?}");
        assert!(
            output.stderr.is_empty(),
            "help should use stdout for {args:?}"
        );
        assert!(
            stdout.contains("Examples:"),
            "examples missing for {args:?}"
        );
        for example in examples {
            assert!(stdout.contains(example), "missing {example:?} for {args:?}");
        }
        if let Some(explanation) = default_explanation {
            assert!(
                stdout.contains(explanation),
                "implicit default missing for {args:?}"
            );
        }
    }
}

#[test]
fn help_flags_are_the_only_help_command_surface() {
    let root_help = run(&["--help"]);
    let meeting_help = run(&["meeting", "--help"]);
    let help_subcommand = run(&["help"]);
    let root_stdout = String::from_utf8(root_help.stdout).expect("help should be UTF-8");
    let meeting_stdout =
        String::from_utf8(meeting_help.stdout).expect("meeting help should be UTF-8");

    assert!(
        !root_stdout
            .lines()
            .any(|line| line.trim_start().starts_with("help "))
    );
    assert!(
        !meeting_stdout
            .lines()
            .any(|line| line.trim_start().starts_with("help "))
    );
    assert_eq!(help_subcommand.status.code(), Some(2));
    assert!(help_subcommand.stdout.is_empty());
}
