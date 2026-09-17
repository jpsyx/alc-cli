//! Soft line wrapping for prose output.
//!
//! Prose aims for [`TARGET_WIDTH`] columns without treating it as a hard edge.
//! A word that only just overruns the target stays put, because a ragged line
//! that ends four columns early reads worse than one that ends four columns
//! late. A word is moved down only when it overruns the tolerance *and* the
//! line it leaves behind is still reasonably full.

use unicode_width::UnicodeWidthStr;

/// The width prose aims for.
const TARGET_WIDTH: usize = 80;
/// The widest line accepted rather than moving a word down.
const TOLERATED_WIDTH: usize = TARGET_WIDTH + 5;
/// The narrowest line accepted when a word is moved down.
const MINIMUM_WIDTH: usize = TARGET_WIDTH - 4;

/// Wraps text into display lines of roughly [`TARGET_WIDTH`] columns.
///
/// Whitespace runs are normalized to single spaces and words are never split,
/// so a word wider than a line simply overhangs it.
#[must_use]
pub fn wrap_prose(text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut current_width = 0;

    for word in text.split_whitespace() {
        let word_width = UnicodeWidthStr::width(word);
        if current.is_empty() {
            current.push_str(word);
            current_width = word_width;
            continue;
        }

        let joined_width = current_width + 1 + word_width;
        if joined_width <= TOLERATED_WIDTH || current_width < MINIMUM_WIDTH {
            current.push(' ');
            current.push_str(word);
            current_width = joined_width;
        } else {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
            current_width = word_width;
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }
    lines
}
