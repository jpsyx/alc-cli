use alc::{MeetingRequest, Request, parse_request, parse_request_with_today};
use chrono::NaiveDate;
use clap::error::ErrorKind;

const fn today() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 9, 6).expect("fixture date should be valid")
}

#[test]
fn meeting_without_a_nested_command_defaults_to_now() {
    let request = parse_request(["alc", "meeting"], today()).expect("request should parse");

    assert!(matches!(request, Request::Meeting(MeetingRequest::Now(_))));
}

#[test]
fn explicit_meeting_now_requests_current_meetings() {
    let request = parse_request(["alc", "meeting", "now"], today()).expect("request should parse");

    assert!(matches!(request, Request::Meeting(MeetingRequest::Now(_))));
}

#[test]
fn meeting_find_accepts_basic_directory_filters() {
    let request = parse_request(
        [
            "alc",
            "meeting",
            "find",
            "turning point",
            "--weekday",
            "sunday",
            "--time",
            "morning",
            "--type",
            "online",
            "--region",
            "brooklyn",
            "--limit",
            "5",
            "--from",
            "10001",
        ],
        today(),
    )
    .expect("request should parse");

    let Request::Meeting(MeetingRequest::Find(filters)) = request else {
        panic!("expected a meeting find request");
    };
    assert_eq!(filters.query(), Some("turning point"));
    assert_eq!(
        filters.weekday().map(|value| value.to_string()).as_deref(),
        Some("sunday")
    );
    assert_eq!(
        filters.time().map(|value| value.to_string()).as_deref(),
        Some("morning")
    );
    assert_eq!(
        filters
            .attendance()
            .map(|value| value.to_string())
            .as_deref(),
        Some("online")
    );
    assert_eq!(filters.region(), Some("brooklyn"));
    assert_eq!(filters.limit(), 5);
    assert_eq!(filters.origin(), Some("10001"));
}

#[test]
fn daily_reflection_defaults_to_today() {
    let request =
        parse_request(["alc", "daily-reflection"], today()).expect("request should parse");

    assert_eq!(request.to_string(), "daily-reflection:2026-09-06");
}

#[test]
fn daily_alias_accepts_a_short_date() {
    let request =
        parse_request(["alc", "daily", "--date", "01-02"], today()).expect("request should parse");

    assert_eq!(request.to_string(), "daily-reflection:2026-01-02");
}

#[test]
fn removed_daily_reflection_flag_is_rejected() {
    let error = parse_request(["alc", "--daily-reflection"], today())
        .expect_err("removed flag should be rejected");

    assert!(
        error
            .to_string()
            .contains("unexpected argument '--daily-reflection'")
    );
}

#[test]
fn help_is_resolved_before_the_date_clock_is_read() {
    let error = parse_request_with_today(["alc", "meeting", "--help"], || {
        panic!("help must not read the date clock")
    })
    .expect_err("help should short-circuit request parsing");

    assert_eq!(error.kind(), ErrorKind::DisplayHelp);
}

#[test]
fn both_meeting_lists_can_disable_the_interactive_pager() {
    let now = parse_request(["alc", "meeting", "now", "--no-pager"], today())
        .expect("now request should parse");
    let find = parse_request(["alc", "meeting", "find", "--no-pager"], today())
        .expect("find request should parse");

    let Request::Meeting(MeetingRequest::Now(now_options)) = now else {
        panic!("expected now options");
    };
    let Request::Meeting(MeetingRequest::Find(find_options)) = find else {
        panic!("expected find options");
    };
    assert!(!now_options.pager_enabled());
    assert!(!find_options.pager_enabled());
}

#[test]
fn both_meeting_lists_enable_the_pager_by_default() {
    let now = parse_request(["alc", "meeting"], today()).expect("now request should parse");
    let find =
        parse_request(["alc", "meeting", "find"], today()).expect("find request should parse");

    let Request::Meeting(MeetingRequest::Now(now_options)) = now else {
        panic!("expected now options");
    };
    let Request::Meeting(MeetingRequest::Find(find_options)) = find else {
        panic!("expected find options");
    };
    assert!(now_options.pager_enabled());
    assert!(find_options.pager_enabled());
}
