use alc::daily_reflection::ReflectionDate;

#[test]
fn parses_supported_reflection_dates() {
    let cases = [
        ("2024-02-29", 2026, "2024-02-29", "02/29"),
        ("09-06", 2026, "2026-09-06", "09/06"),
    ];

    for (input, current_year, expected_date, expected_api_path) in cases {
        let date = ReflectionDate::parse(input, current_year).expect("date should be valid");

        assert_eq!(date.to_string(), expected_date);
        assert_eq!(date.api_path(), expected_api_path);
    }
}

#[test]
fn rejects_invalid_reflection_dates() {
    for input in ["2026-02-29", "2026-02-30", "09/06", "09-6"] {
        let error = ReflectionDate::parse(input, 2026).expect_err("date should be invalid");

        assert_eq!(
            error.to_string(),
            format!("invalid date '{input}'; expected YYYY-MM-DD or MM-DD")
        );
    }
}
