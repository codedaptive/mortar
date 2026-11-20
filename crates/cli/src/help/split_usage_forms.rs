//! split_usage_forms: a usage line as one line per form of its first top-level "( a | b )" group.

/// One line per top-level alternative of `usage`'s first "(" group that is outside every bracket: the text before
/// the "(", the alternative trimmed, and the text after its ")", joined by single spaces (an empty part adds no
/// space). A " | " inside a "[..]" or a nested "(..)" does not split. A usage with no such group, or a group of one
/// alternative, is one line, unchanged.
pub fn split_usage_forms(usage: &str) -> Vec<String> {
    let Some((open, close)) = first_group(usage) else {
        return vec![usage.to_string()];
    };
    let alternatives = top_level_alternatives(&usage[open + 1..close]);
    if alternatives.len() < 2 {
        return vec![usage.to_string()];
    }
    let before = usage[..open].trim();
    let after = usage[close + 1..].trim();
    alternatives
        .into_iter()
        .map(|alternative| {
            [before, alternative.trim(), after].into_iter().filter(|part| !part.is_empty()).collect::<Vec<_>>().join(" ")
        })
        .collect()
}

/// The byte offsets of the first "(" outside any bracket and its matching ")".
fn first_group(usage: &str) -> Option<(usize, usize)> {
    let mut depth = 0usize;
    let mut open = None;
    for (index, character) in usage.char_indices() {
        match character {
            '(' | '[' => {
                if character == '(' && depth == 0 {
                    open = Some(index);
                }
                depth += 1;
            }
            ')' | ']' => {
                depth = depth.saturating_sub(1);
                if let (0, Some(start)) = (depth, open) {
                    return Some((start, index));
                }
            }
            _ => {}
        }
    }
    None
}

/// `inner` split at each " | " that is outside every "[..]" and "(..)".
fn top_level_alternatives(inner: &str) -> Vec<&str> {
    let mut depth = 0usize;
    let mut start = 0;
    let mut parts = Vec::new();
    for (index, character) in inner.char_indices() {
        match character {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            '|' if depth == 0 && index > 0 && inner[index - 1..].starts_with(" | ") => {
                parts.push(&inner[start..index - 1]);
                start = index + 2;
            }
            _ => {}
        }
    }
    parts.push(&inner[start..]);
    parts
}
