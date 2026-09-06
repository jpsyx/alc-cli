use alc::{command_with_theme, theme::Theme};

#[test]
fn dark_theme_uses_bright_semantic_colors() {
    let theme = Theme::dark(true);

    assert_eq!(theme.heading("heading"), "\x1b[1;95mheading\x1b[0m");
    assert_eq!(theme.accent("accent"), "\x1b[96maccent\x1b[0m");
    assert_eq!(theme.value("value"), "\x1b[97mvalue\x1b[0m");
    assert_eq!(theme.muted("muted"), "\x1b[90mmuted\x1b[0m");
    assert_eq!(theme.success("success"), "\x1b[92msuccess\x1b[0m");
    assert_eq!(theme.warning("warning"), "\x1b[93mwarning\x1b[0m");
    assert_eq!(theme.error("error"), "\x1b[91merror\x1b[0m");
    assert_eq!(theme.info("info"), "\x1b[94minfo\x1b[0m");
    assert_eq!(theme.prompt("prompt"), "\x1b[1;96mprompt\x1b[0m");
    assert_eq!(theme.matched("match"), "\x1b[1;30;103mmatch\x1b[0m");
    assert_eq!(
        theme.in_progress("IN PROGRESS"),
        "\x1b[1;92mIN PROGRESS\x1b[0m"
    );
    assert_eq!(
        theme.time_soon("[In 15 minutes]"),
        "\x1b[1;93m[In 15 minutes]\x1b[0m"
    );
    assert_eq!(
        theme.time_started("[Started 15 minutes ago]"),
        "\x1b[91m[Started 15 minutes ago]\x1b[0m"
    );
    assert_eq!(theme.access_online("[Online]"), "\x1b[1;92m[Online]\x1b[0m");
    assert_eq!(theme.access_hybrid("[Hybrid]"), "\x1b[1;94m[Hybrid]\x1b[0m");
    assert_eq!(
        theme.access_in_person("[In person]"),
        "\x1b[1;96m[In person]\x1b[0m"
    );
}

#[test]
fn disabled_theme_preserves_plain_pipeable_text() {
    let theme = Theme::dark(false);

    for rendered in [
        theme.heading("heading"),
        theme.accent("accent"),
        theme.value("value"),
        theme.muted("muted"),
        theme.success("success"),
        theme.warning("warning"),
        theme.error("error"),
        theme.info("info"),
        theme.prompt("prompt"),
        theme.matched("match"),
        theme.in_progress("IN PROGRESS"),
        theme.time_soon("[In 15 minutes]"),
        theme.time_started("[Started 15 minutes ago]"),
        theme.access_online("[Online]"),
        theme.access_hybrid("[Hybrid]"),
        theme.access_in_person("[In person]"),
    ] {
        assert!(!rendered.contains('\x1b'));
    }
    assert_eq!(theme.error_line("✗", "problem"), "✗ problem");
}

#[test]
fn clap_help_uses_the_same_theme_and_can_remain_plain() {
    let colored_help = command_with_theme(Theme::dark(true)).render_help();
    let colored = colored_help.ansi().to_string();
    let plain = command_with_theme(Theme::dark(false))
        .render_help()
        .to_string();

    assert!(
        colored.contains("\x1b[1m\x1b[95mUsage:\x1b[0m"),
        "unexpected colored help: {colored:?}"
    );
    assert!(
        colored.contains("\x1b[96mmeeting\x1b[0m"),
        "unexpected colored help: {colored:?}"
    );
    assert!(!plain.contains('\x1b'));
    assert!(plain.contains("Usage: alc <COMMAND>"));
}
