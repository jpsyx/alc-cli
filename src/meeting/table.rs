use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::theme::Theme;

pub(super) fn contains_case_insensitive(value: &str, query: &str) -> bool {
    value.to_lowercase().contains(&query.to_lowercase())
}

pub(super) fn case_insensitive_match_ranges(
    value: &str,
    query: &str,
) -> Vec<std::ops::Range<usize>> {
    let query = query.to_lowercase();
    if query.is_empty() {
        return Vec::new();
    }

    let mut lowercase = String::new();
    let mut original_starts = Vec::new();
    let mut original_ends = Vec::new();
    let mut characters = value.char_indices().peekable();
    while let Some((start, character)) = characters.next() {
        let end = characters.peek().map_or(value.len(), |(index, _)| *index);
        let folded = character.to_lowercase().collect::<String>();
        original_starts.extend(std::iter::repeat_n(start, folded.len()));
        original_ends.extend(std::iter::repeat_n(end, folded.len()));
        lowercase.push_str(&folded);
    }

    lowercase
        .match_indices(&query)
        .filter_map(|(start, matched)| {
            let end = start + matched.len();
            Some(*original_starts.get(start)?..*original_ends.get(end.checked_sub(1)?)?)
        })
        .collect()
}

pub(super) fn border(
    left: char,
    junction: char,
    right: char,
    label_width: usize,
    value_width: usize,
    theme: Theme,
) -> String {
    theme.muted(&format!(
        "{left}{}{junction}{}{right}",
        "─".repeat(label_width + 2),
        "─".repeat(value_width + 2)
    ))
}

pub(super) fn pad(value: &str, width: usize) -> String {
    let padding = width.saturating_sub(UnicodeWidthStr::width(value));
    format!("{value}{}", " ".repeat(padding))
}

pub(super) fn truncate_to_width(value: &str, width: usize) -> String {
    if UnicodeWidthStr::width(value) <= width {
        return value.to_owned();
    }
    if width == 0 {
        return String::new();
    }

    let content_width = width - 1;
    let mut rendered = String::new();
    let mut rendered_width = 0;
    for character in value.chars() {
        let character_width = UnicodeWidthChar::width(character).unwrap_or(0);
        if rendered_width + character_width > content_width {
            break;
        }
        rendered.push(character);
        rendered_width += character_width;
    }
    rendered.push('…');
    rendered
}

pub(super) fn wrap_text(value: &str, width: usize) -> Vec<String> {
    let value = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in value.split(' ') {
        let word_width = UnicodeWidthStr::width(word);
        if !current.is_empty() && UnicodeWidthStr::width(current.as_str()) + 1 + word_width > width
        {
            lines.push(std::mem::take(&mut current));
        }
        if word_width > width {
            if !current.is_empty() {
                lines.push(std::mem::take(&mut current));
            }
            let mut chunk = String::new();
            let mut chunk_width = 0;
            for character in word.chars() {
                let character_width = UnicodeWidthChar::width(character).unwrap_or(0);
                if !chunk.is_empty() && chunk_width + character_width > width {
                    lines.push(std::mem::take(&mut chunk));
                    chunk_width = 0;
                }
                chunk.push(character);
                chunk_width += character_width;
                if chunk_width == width {
                    lines.push(std::mem::take(&mut chunk));
                    chunk_width = 0;
                }
            }
            current = chunk;
        } else {
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    if lines.is_empty() {
        lines.push(String::new());
    }
    lines
}
