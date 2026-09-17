use alc::text_wrap::wrap_prose;

fn word(width: usize) -> String {
    "x".repeat(width)
}

#[test]
fn keeps_a_short_paragraph_on_one_line() {
    assert_eq!(
        wrap_prose("A short reflection line."),
        ["A short reflection line."]
    );
}

#[test]
fn wraps_prose_near_the_target_width() {
    let paragraph = "When I am disturbed, it is because I find some person, place, thing, or situation, some fact of my life, unacceptable to me, and I can find no serenity until I accept that person, place, thing, or situation as being exactly the way it is supposed to be at this moment.";

    let lines = wrap_prose(paragraph);

    assert!(lines.len() > 1);
    assert_eq!(lines.join(" "), paragraph);
    for line in &lines {
        assert!(line.chars().count() <= 85, "line too long: {line}");
    }
}

#[test]
fn keeps_a_word_that_ends_just_past_the_target() {
    // 76 + 1 + 7 = 84 characters, inside the tolerance above 80.
    let paragraph = format!("{} {}", word(76), word(7));

    assert_eq!(wrap_prose(&paragraph), std::slice::from_ref(&paragraph));
}

#[test]
fn moves_a_word_that_overshoots_the_tolerance() {
    // 76 + 1 + 9 = 86 characters, and the line still holds 76 without it.
    let paragraph = format!("{} {}", word(76), word(9));

    assert_eq!(wrap_prose(&paragraph), [word(76), word(9)]);
}

#[test]
fn keeps_an_overshooting_word_that_would_leave_the_line_too_short() {
    // 75 + 1 + 10 = 86 characters, but breaking would leave a 75-character line.
    let paragraph = format!("{} {}", word(75), word(10));

    assert_eq!(wrap_prose(&paragraph), std::slice::from_ref(&paragraph));
}

#[test]
fn never_splits_a_word_that_is_longer_than_the_line() {
    let url = "https://www.aa.org/daily-reflections?date=00-09-06&extra=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let paragraph = format!("See {url} today.");

    let lines = wrap_prose(&paragraph);

    // The word cannot fit on any line, so it is never broken, and gluing it to
    // the short line ahead of it costs nothing the line was not already over.
    assert_eq!(lines, [format!("See {url}"), "today.".to_owned()]);
}

#[test]
fn normalizes_runs_of_whitespace() {
    assert_eq!(wrap_prose("  keep   it\tsimple \n"), ["keep it simple"]);
}

#[test]
fn returns_no_lines_for_empty_text() {
    assert!(wrap_prose("   ").is_empty());
}

#[test]
fn measures_display_width_rather_than_bytes() {
    // 80 + 1 + 4 columns is exactly the tolerance; counting bytes would wrap.
    let paragraph = format!("{} café", word(80));

    assert_eq!(wrap_prose(&paragraph), std::slice::from_ref(&paragraph));
}
