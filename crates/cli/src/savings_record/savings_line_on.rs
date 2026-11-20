//! The user's switch for the "[mr] tokens saved" line.

use super::top_level_bool;

/// False only when `config` (the text of the user's config.json) has a top-level "savings_line" member set to
/// false. No file (None), no such member, true, a value that is not a boolean, or text that is not one JSON
/// object: true, so a broken file never hides the line or fails the verb.
pub fn savings_line_on(config: Option<&str>) -> bool {
    config.is_none_or(|text| top_level_bool(text, "savings_line") != Ok(Some(false)))
}
