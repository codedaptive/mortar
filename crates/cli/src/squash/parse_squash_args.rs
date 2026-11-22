//! The words of `mr squash` after its globals, read into the zone it squashes and the caller's message.

use crate::config::Config;
use crate::landing::known_zone;

/// One zone word, a zone the config names, and at most one `--message <text>` or `--message=<text>`, before or
/// after the zone; the text is kept as given. The Err is the reason, the first in line order: --message with no
/// word after it, a second --message, a text empty or only whitespace, a second plain word (named); then no zone
/// word, or known_zone's reason for a zone the config does not name. A dashed word other than --message is a
/// plain word.
pub fn parse_squash_args(rest: &[String], config: &Config) -> Result<(String, Option<String>), String> {
    let mut zone: Option<&String> = None;
    let mut message: Option<String> = None;
    let mut words = rest.iter();
    while let Some(word) = words.next() {
        let text = match word.strip_prefix("--message=") {
            Some(text) => text,
            None if word == "--message" => words.next().map(String::as_str).ok_or("--message needs a text")?,
            None if zone.is_some() => return Err(format!("unexpected word {word:?} after the zone")),
            None => {
                zone = Some(word);
                continue;
            }
        };
        if message.is_some() {
            return Err("--message is given twice".to_string());
        }
        if text.trim().is_empty() {
            return Err("--message needs a text that is not empty or only whitespace".to_string());
        }
        message = Some(text.to_string());
    }
    let Some(zone) = zone else {
        return Err("squash needs a zone".to_string());
    };
    known_zone(zone, config)?;
    Ok((zone.clone(), message))
}
