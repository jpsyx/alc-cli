use alc::{MeetingRequest, Request, meeting, parse_request, theme::Theme};
use chrono::{NaiveDate, TimeZone};
use chrono_tz::America::New_York;

const DIRECTORY: &str = r#"[
  {
    "id": 1,
    "name": "Morning Light Hall",
    "slug": "morning-light-hall",
    "url": "https://www.nyintergroup.org/meetings/morning-light-hall/",
    "day": 0,
    "time": "09:00",
    "end_time": "10:00",
    "types": ["O", "X"],
    "location": "Example Church",
    "formatted_address": "10 Example St, Brooklyn, NY 11201, USA",
    "regions": ["Brooklyn", "Downtown Brooklyn"],
    "attendance_option": "in_person"
  },
  {
    "id": 2,
    "name": "Morning Light Hybrid",
    "slug": "morning-light-hybrid",
    "url": "https://www.nyintergroup.org/meetings/morning-light-hybrid/",
    "day": 0,
    "time": "09:30",
    "end_time": "10:30",
    "types": ["D", "ONL"],
    "conference_url": "https://zoom.us/j/123456789",
    "conference_url_notes": "Meeting ID: 123 456 789",
    "conference_phone": "+1 212 555 0199",
    "conference_phone_notes": "Phone passcode: 456",
    "location": "Example Hall",
    "formatted_address": "20 Example Ave, Brooklyn, NY 11201, USA",
    "regions": ["Brooklyn", "Downtown Brooklyn"],
    "attendance_option": "hybrid"
  },
  {
    "id": 3,
    "name": "Morning Light Online",
    "slug": "morning-light-online",
    "url": "https://www.nyintergroup.org/meetings/morning-light-online/",
    "day": 0,
    "time": "10:00",
    "end_time": "11:00",
    "types": ["O", "ONL"],
    "conference_url": "https://meet.example.test/room",
    "conference_phone": "+1 212 555 0100",
    "group": "Morning Light Group",
    "formatted_address": "Brooklyn, NY, USA",
    "regions": ["Brooklyn"],
    "attendance_option": "online"
  },
  {
    "id": 4,
    "name": "Later Online",
    "slug": "later-online",
    "url": "https://www.nyintergroup.org/meetings/later-online/",
    "day": 0,
    "time": "11:00",
    "end_time": "12:00",
    "types": ["ONL"],
    "conference_url": "https://meet.example.test/later",
    "formatted_address": "New York, NY, USA",
    "regions": ["Manhattan"],
    "attendance_option": "online"
  },
  {
    "id": 5,
    "name": "Inactive Listing",
    "slug": "inactive-listing",
    "url": "https://www.nyintergroup.org/meetings/inactive-listing/",
    "day": 0,
    "time": "09:15",
    "end_time": "10:15",
    "types": ["TC"],
    "formatted_address": "Brooklyn, NY, USA",
    "regions": ["Brooklyn"],
    "attendance_option": "inactive"
  }
]"#;

const fn today() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 9, 6).expect("fixture date should be valid")
}

#[test]
fn parses_access_details_from_the_directory_feed() {
    let meetings = meeting::parse_directory(DIRECTORY).expect("directory should parse");

    assert_eq!(meetings.len(), 5);
    assert_eq!(meetings[1].name(), "Morning Light Hybrid");
    assert!(meetings[1].is_online());
    assert!(meetings[1].is_in_person());
    assert_eq!(meetings[1].join_url(), Some("https://zoom.us/j/123456789"));
    assert_eq!(
        meetings[1].source_url(),
        "https://www.nyintergroup.org/meetings/morning-light-hybrid/"
    );
}

#[test]
fn find_combines_filters_and_keeps_hybrid_in_online_results() {
    let meetings = meeting::parse_directory(DIRECTORY).expect("directory should parse");
    let request = parse_request(
        [
            "alc",
            "meeting",
            "find",
            "morning light",
            "--weekday",
            "sunday",
            "--time",
            "morning",
            "--type",
            "online",
            "--region",
            "brooklyn",
        ],
        today(),
    )
    .expect("request should parse");
    let Request::Meeting(MeetingRequest::Find(filters)) = request else {
        panic!("expected find filters");
    };

    let matches = meeting::find(&meetings, &filters);
    let names = matches
        .iter()
        .map(|meeting| meeting.name())
        .collect::<Vec<_>>();

    assert_eq!(names, ["Morning Light Hybrid", "Morning Light Online"]);
}

#[test]
fn find_ranks_online_capable_meetings_before_in_person_meetings() {
    let meetings = meeting::parse_directory(DIRECTORY).expect("directory should parse");
    let request = parse_request(["alc", "meeting", "find", "--weekday", "sunday"], today())
        .expect("request should parse");
    let Request::Meeting(MeetingRequest::Find(filters)) = request else {
        panic!("expected find filters");
    };

    let matches = meeting::find(&meetings, &filters);
    let names = matches
        .iter()
        .map(|meeting| meeting.name())
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        [
            "Morning Light Hybrid",
            "Morning Light Online",
            "Later Online",
            "Morning Light Hall"
        ]
    );
}

#[test]
fn now_returns_in_progress_and_next_hour_with_online_first() {
    let meetings = meeting::parse_directory(DIRECTORY).expect("directory should parse");
    let request = parse_request(["alc", "meeting", "now"], today()).expect("request should parse");
    let Request::Meeting(MeetingRequest::Now(options)) = request else {
        panic!("expected now options");
    };
    let now = New_York
        .with_ymd_and_hms(2026, 9, 6, 9, 45, 0)
        .single()
        .expect("fixture time should exist");

    let matches = meeting::now(&meetings, now, &options);
    let names = matches
        .iter()
        .map(|item| item.meeting().name())
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        [
            "Morning Light Hybrid",
            "Morning Light Online",
            "Morning Light Hall"
        ]
    );
    assert_eq!(matches[0].availability().to_string(), "in progress");
    assert_eq!(matches[1].availability().to_string(), "starts in 15m");
}

#[test]
fn directions_link_can_include_an_explicit_origin() {
    let meetings = meeting::parse_directory(DIRECTORY).expect("directory should parse");

    let directions = meeting::directions_url(&meetings[0], Some("New York, NY"))
        .expect("in-person meeting should have directions");

    assert_eq!(
        directions,
        "https://www.google.com/maps/dir/?api=1&origin=New+York%2C+NY&destination=10+Example+St%2C+Brooklyn%2C+NY+11201%2C+USA"
    );
    assert_eq!(meeting::directions_url(&meetings[2], None), None);
}

#[test]
fn renders_online_and_in_person_actions_with_source_links() {
    let meetings = meeting::parse_directory(DIRECTORY).expect("directory should parse");
    let request = parse_request(
        [
            "alc",
            "meeting",
            "find",
            "morning light hybrid",
            "--from",
            "New York, NY",
        ],
        today(),
    )
    .expect("request should parse");
    let Request::Meeting(MeetingRequest::Find(filters)) = request else {
        panic!("expected find filters");
    };
    let matches = meeting::find(&meetings, &filters);

    let output = meeting::render_find(&matches, filters.origin(), Theme::dark(false));

    assert!(output.starts_with("1 meeting found (online-capable first).\n\n"));
    assert!(output.contains("┌────────────────┬"));
    assert!(output.contains("│ Meeting        │ Morning Light Hybrid"));
    assert!(output.contains("│ Access         │ Hybrid"));
    assert!(output.contains("│ Schedule       │ Sunday 9:30 AM to 10:30 AM"));
    assert!(output.contains("│ Join           │ https://zoom.us/j/123456789"));
    assert!(output.contains("│ Online details │ Meeting ID: 123 456 789"));
    assert!(output.contains("│ Phone          │ +1 212 555 0199"));
    assert!(output.contains("│ Phone details  │ Phone passcode: 456"));
    assert!(output.contains("│ Address        │ 20 Example Ave, Brooklyn, NY 11201, USA"));
    assert!(output.contains(
        "│ Directions     │ https://www.google.com/maps/dir/?api=1&origin=New+York%2C+NY"
    ));
    assert!(output.contains(
        "│ Source         │ https://www.nyintergroup.org/meetings/morning-light-hybrid/"
    ));
    assert!(output.contains("└────────────────┴"));
}

#[test]
fn meeting_output_uses_semantic_terminal_styles() {
    let meetings = meeting::parse_directory(DIRECTORY).expect("directory should parse");
    let request = parse_request(["alc", "meeting", "find", "morning light hybrid"], today())
        .expect("request should parse");
    let Request::Meeting(MeetingRequest::Find(filters)) = request else {
        panic!("expected find filters");
    };

    let output = meeting::render_find(&meeting::find(&meetings, &filters), None, Theme::dark(true));

    assert!(output.contains("\x1b[1;95m1 meeting found (online-capable first).\x1b[0m"));
    assert!(output.contains("\x1b[94mHybrid"));
    assert!(output.contains("\x1b[97mMorning Light Hybrid"));
    assert!(output.contains("\x1b[96mJoin"));
    assert!(output.contains("\x1b[97mhttps://zoom.us/j/123456789"));
}

#[test]
fn renders_current_availability_in_new_york_time() {
    let meetings = meeting::parse_directory(DIRECTORY).expect("directory should parse");
    let request = parse_request(["alc", "meeting", "now"], today()).expect("request should parse");
    let Request::Meeting(MeetingRequest::Now(options)) = request else {
        panic!("expected now options");
    };
    let at = New_York
        .with_ymd_and_hms(2026, 9, 6, 9, 45, 0)
        .single()
        .expect("fixture time should exist");
    let matches = meeting::now(&meetings, at, &options);

    let output = meeting::render_now(&matches, options.origin(), Theme::dark(false));

    assert!(output.starts_with("3 meetings available now (New York time).\n\n"));
    assert!(output.contains("│ Access         │ Hybrid"));
    assert!(output.contains("│ Schedule       │ Sunday 9:30 AM to 10:30 AM"));
    assert!(output.contains("│ Status         │ in progress"));
    assert!(output.contains("│ Access         │ Online"));
    assert!(output.contains("│ Status         │ starts in 15m"));
}

#[test]
fn renders_an_explicit_empty_result() {
    assert_eq!(
        meeting::render_find(&[], None, Theme::dark(false)),
        "No meetings found. Try broader filters.\n"
    );
}

#[test]
fn decodes_html_entities_stored_inside_json_strings() {
    let directory = r#"[{
      "id": 99,
      "name": "Rock &amp; Roll",
      "slug": "rock-roll",
      "url": "https://www.nyintergroup.org/meetings/rock-roll/",
      "day": 0,
      "time": "09:00",
      "end_time": "10:00",
      "regions": ["Chelsea &amp; Gramercy"],
      "attendance_option": "online"
    }]"#;

    let meetings = meeting::parse_directory(directory).expect("directory should parse");
    let request = parse_request(["alc", "meeting", "find"], today()).expect("request should parse");
    let Request::Meeting(MeetingRequest::Find(filters)) = request else {
        panic!("expected find filters");
    };

    let output = meeting::render_find(
        &meeting::find(&meetings, &filters),
        None,
        Theme::dark(false),
    );

    assert_eq!(meetings[0].name(), "Rock & Roll");
    assert!(output.contains("│ Region         │ Chelsea & Gramercy"));
}
