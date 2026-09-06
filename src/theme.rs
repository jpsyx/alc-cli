//! Semantic terminal colors for consistent human-facing output.

use std::io::IsTerminal;

use clap::{
    ColorChoice,
    builder::styling::{AnsiColor, Styles},
};

/// A dark-terminal palette whose color emission can be disabled.
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    color: bool,
    heading_code: &'static str,
    accent_code: &'static str,
    value_code: &'static str,
    muted_code: &'static str,
    success_code: &'static str,
    warning_code: &'static str,
    error_code: &'static str,
    info_code: &'static str,
    prompt_code: &'static str,
    matched_code: &'static str,
    in_progress_code: &'static str,
    time_soon_code: &'static str,
    access_online_code: &'static str,
    access_hybrid_code: &'static str,
    access_in_person_code: &'static str,
}

impl Theme {
    /// Creates the dark-terminal palette with optional ANSI output.
    #[must_use]
    pub const fn dark(color: bool) -> Self {
        Self {
            color,
            heading_code: "1;95",
            accent_code: "96",
            value_code: "97",
            muted_code: "90",
            success_code: "92",
            warning_code: "93",
            error_code: "91",
            info_code: "94",
            prompt_code: "1;96",
            matched_code: "1;30;103",
            in_progress_code: "1;92",
            time_soon_code: "1;93",
            access_online_code: "1;92",
            access_hybrid_code: "1;94",
            access_in_person_code: "1;96",
        }
    }

    /// Creates the active theme, enabling color only for an interactive terminal.
    #[must_use]
    pub fn active() -> Self {
        Self::dark(color_enabled())
    }

    /// Styles a section heading.
    #[must_use]
    pub fn heading(self, value: &str) -> String {
        self.paint(self.heading_code, value)
    }

    /// Styles a command, key, or label.
    #[must_use]
    pub fn accent(self, value: &str) -> String {
        self.paint(self.accent_code, value)
    }

    /// Styles a primary value.
    #[must_use]
    pub fn value(self, value: &str) -> String {
        self.paint(self.value_code, value)
    }

    /// Styles secondary text.
    #[must_use]
    pub fn muted(self, value: &str) -> String {
        self.paint(self.muted_code, value)
    }

    /// Styles a successful result.
    #[must_use]
    pub fn success(self, value: &str) -> String {
        self.paint(self.success_code, value)
    }

    /// Styles a warning.
    #[must_use]
    pub fn warning(self, value: &str) -> String {
        self.paint(self.warning_code, value)
    }

    /// Styles an error.
    #[must_use]
    pub fn error(self, value: &str) -> String {
        self.paint(self.error_code, value)
    }

    /// Styles a compact error icon and message.
    #[must_use]
    pub fn error_line(self, icon: &str, message: &str) -> String {
        format!("{} {}", self.error(icon), self.error(message))
    }

    /// Styles informational text.
    #[must_use]
    pub fn info(self, value: &str) -> String {
        self.paint(self.info_code, value)
    }

    /// Styles an interactive prompt.
    #[must_use]
    pub fn prompt(self, value: &str) -> String {
        self.paint(self.prompt_code, value)
    }

    /// Highlights text matched by an interactive filter.
    #[must_use]
    pub fn matched(self, value: &str) -> String {
        self.paint(self.matched_code, value)
    }

    /// Styles an uppercase in-progress meeting status.
    #[must_use]
    pub fn in_progress(self, value: &str) -> String {
        self.paint(self.in_progress_code, value)
    }

    /// Styles a relative start time for a meeting beginning soon.
    #[must_use]
    pub fn time_soon(self, value: &str) -> String {
        self.paint(self.time_soon_code, value)
    }

    /// Styles elapsed time for a meeting already underway.
    #[must_use]
    pub fn time_started(self, value: &str) -> String {
        self.paint(self.error_code, value)
    }

    /// Styles an online access badge.
    #[must_use]
    pub fn access_online(self, value: &str) -> String {
        self.paint(self.access_online_code, value)
    }

    /// Styles a hybrid access badge.
    #[must_use]
    pub fn access_hybrid(self, value: &str) -> String {
        self.paint(self.access_hybrid_code, value)
    }

    /// Styles an in-person access badge.
    #[must_use]
    pub fn access_in_person(self, value: &str) -> String {
        self.paint(self.access_in_person_code, value)
    }

    pub(crate) const fn clap_styles(self) -> Styles {
        if !self.color {
            return Styles::plain();
        }

        Styles::styled()
            .header(AnsiColor::BrightMagenta.on_default().bold())
            .usage(AnsiColor::BrightMagenta.on_default().bold())
            .error(AnsiColor::BrightRed.on_default().bold())
            .literal(AnsiColor::BrightCyan.on_default())
            .placeholder(AnsiColor::BrightWhite.on_default())
            .valid(AnsiColor::BrightGreen.on_default())
            .invalid(AnsiColor::BrightYellow.on_default())
            .context(AnsiColor::BrightBlack.on_default())
            .context_value(AnsiColor::BrightWhite.on_default())
    }

    pub(crate) const fn clap_color_choice(self) -> ColorChoice {
        if self.color {
            ColorChoice::Always
        } else {
            ColorChoice::Never
        }
    }

    fn paint(self, code: &str, value: &str) -> String {
        if self.color {
            format!("\x1b[{code}m{value}\x1b[0m")
        } else {
            value.to_owned()
        }
    }
}

fn color_enabled() -> bool {
    should_use_color(
        std::io::stderr().is_terminal(),
        std::env::var_os("NO_COLOR").is_some(),
    )
}

const fn should_use_color(stderr_is_terminal: bool, no_color_is_set: bool) -> bool {
    stderr_is_terminal && !no_color_is_set
}

#[cfg(test)]
mod tests {
    use super::should_use_color;

    #[test]
    fn color_requires_a_terminal_and_no_no_color_setting() {
        assert!(should_use_color(true, false));
        assert!(!should_use_color(false, false));
        assert!(!should_use_color(true, true));
        assert!(!should_use_color(false, true));
    }
}
