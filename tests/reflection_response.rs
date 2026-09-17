use std::{
    io::{Read, Write},
    net::TcpListener,
    thread,
};

use alc::daily_reflection::{ReflectionDate, fetch_from, parse_api_response, render};
use alc::theme::Theme;

const API_RESPONSE: &str = r#"{
  "method": "GET",
  "err": 200,
  "data": "<article data-date=\"09-06\"><h3><span class=\"field--name-title\">KEEP IT SIMPLE &amp; TRUE</span></h3><div class=\"field--name-field-date\">September 06</div><div class=\"field--name-body\"><p><strong>A short quotation.</strong></p><p>EXAMPLE BOOK, p. 1</p><p>Reflection text with care &amp; clarity.</p></div><div class=\"field--name-field-copyright\"><div class=\"field--name-description\"><p>Example copyright notice.</p></div></div></article>",
  "errMedia": 404,
  "dataMedia": ""
}"#;

#[test]
fn parses_the_official_api_response_into_plain_text() {
    let date = ReflectionDate::parse("2026-09-06", 2026).expect("fixture date should parse");

    let reflection = parse_api_response(API_RESPONSE, date).expect("response should parse");

    assert_eq!(reflection.title(), "KEEP IT SIMPLE & TRUE");
    assert_eq!(reflection.date_label(), "September 06");
    assert_eq!(
        reflection.paragraphs(),
        [
            "A short quotation.",
            "EXAMPLE BOOK, p. 1",
            "Reflection text with care & clarity."
        ]
    );
    assert_eq!(reflection.copyright(), "Example copyright notice.");
    assert_eq!(
        reflection.source_url(),
        "https://www.aa.org/daily-reflections?date=00-09-06"
    );
}

#[test]
fn rejects_an_api_response_for_the_wrong_date() {
    let date = ReflectionDate::parse("2026-09-07", 2026).expect("fixture date should parse");

    let error = parse_api_response(API_RESPONSE, date).expect_err("wrong date should be rejected");

    assert_eq!(
        error.to_string(),
        "AA.org returned reflection 09-06 when 09-07 was requested"
    );
}

#[test]
fn renders_a_reflection_with_source_and_copyright() {
    let date = ReflectionDate::parse("2026-09-06", 2026).expect("fixture date should parse");
    let reflection = parse_api_response(API_RESPONSE, date).expect("response should parse");

    let output = render(&reflection, Theme::dark(false));

    assert_eq!(
        output,
        "Daily Reflection | September 06\n\nKEEP IT SIMPLE & TRUE\n\nA short quotation.\n\nEXAMPLE BOOK, p. 1\n\nReflection text with care & clarity.\n\nSource: https://www.aa.org/daily-reflections?date=00-09-06\nExample copyright notice.\n"
    );
}

#[test]
fn reflection_output_uses_semantic_terminal_styles() {
    let date = ReflectionDate::parse("2026-09-06", 2026).expect("fixture date should parse");
    let reflection = parse_api_response(API_RESPONSE, date).expect("response should parse");

    let output = render(&reflection, Theme::dark(true));

    assert!(output.contains("\x1b[1;95mDaily Reflection | September 06\x1b[0m"));
    assert!(output.contains("\x1b[97mKEEP IT SIMPLE & TRUE\x1b[0m"));
    assert!(output.contains("\x1b[96mSource:\x1b[0m \x1b[97mhttps://www.aa.org/"));
    assert!(output.contains("\x1b[90mExample copyright notice.\x1b[0m"));
}

#[test]
fn fetches_the_requested_date_from_a_reflection_api() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("test server should bind");
    let address = listener
        .local_addr()
        .expect("test server should have an address");
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("test server should accept");
        let mut request = [0_u8; 1024];
        let bytes_read = stream
            .read(&mut request)
            .expect("request should be readable");
        let request = String::from_utf8_lossy(&request[..bytes_read]);
        assert!(request.starts_with("GET /09/06 HTTP/1.1\r\n"));

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{API_RESPONSE}",
            API_RESPONSE.len()
        );
        stream
            .write_all(response.as_bytes())
            .expect("response should be writable");
    });
    let date = ReflectionDate::parse("2026-09-06", 2026).expect("fixture date should parse");

    let reflection =
        fetch_from(&format!("http://{address}"), date).expect("reflection should be fetched");
    server.join().expect("test server should finish");

    assert_eq!(reflection.title(), "KEEP IT SIMPLE & TRUE");
}

const LONG_API_RESPONSE: &str = r#"{
  "method": "GET",
  "err": 200,
  "data": "<article data-date=\"09-06\"><h3><span class=\"field--name-title\">KEEP IT SIMPLE</span></h3><div class=\"field--name-field-date\">September 06</div><div class=\"field--name-body\"><p>When I am disturbed, it is because I find some person, place, thing, or situation, some fact of my life, unacceptable to me, and I can find no serenity until I accept that person as being exactly the way it is supposed to be at this moment.</p></div><div class=\"field--name-field-copyright\"><div class=\"field--name-description\"><p>Example copyright notice that runs well past the comfortable reading width of a terminal window.</p></div></div></article>",
  "errMedia": 404,
  "dataMedia": ""
}"#;

#[test]
fn wraps_long_reflection_prose_for_comfortable_reading() {
    let date = ReflectionDate::parse("2026-09-06", 2026).expect("fixture date should parse");
    let reflection = parse_api_response(LONG_API_RESPONSE, date).expect("response should parse");

    let output = render(&reflection, Theme::dark(false));

    let body = output
        .split("\n\n")
        .nth(2)
        .expect("output should contain a body block");
    assert_eq!(
        body.lines().collect::<Vec<_>>(),
        [
            "When I am disturbed, it is because I find some person, place, thing, or situation,",
            "some fact of my life, unacceptable to me, and I can find no serenity until I accept",
            "that person as being exactly the way it is supposed to be at this moment.",
        ]
    );
    assert!(
        output.lines().any(|line| line
            == "Example copyright notice that runs well past the comfortable reading width of a"),
        "copyright should wrap: {output}"
    );
}

#[test]
fn keeps_the_source_url_on_one_line() {
    let date = ReflectionDate::parse("2026-09-06", 2026).expect("fixture date should parse");
    let reflection = parse_api_response(LONG_API_RESPONSE, date).expect("response should parse");

    let output = render(&reflection, Theme::dark(false));

    assert!(
        output
            .lines()
            .any(|line| line == "Source: https://www.aa.org/daily-reflections?date=00-09-06"),
        "source URL should not wrap: {output}"
    );
}
