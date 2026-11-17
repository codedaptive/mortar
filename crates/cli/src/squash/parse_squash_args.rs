//! The words of `mr squash` after its globals, read into the zone it squashes.

use crate::config::Config;

/// Exactly one word, a zone the config names. The Err is the reason: no word, a second word (named), or a word that
/// is not one of the config's zones (named, with the config's zones in name order).
pub fn parse_squash_args(rest: &[String], config: &Config) -> Result<String, String> {
    let Some((zone, words)) = rest.split_first() else {
        return Err("squash needs a zone".to_string());
    };
    if let Some(extra) = words.first() {
        return Err(format!("unexpected word {extra:?} after the zone"));
    }
    if !config.zones.contains_key(zone) {
        let names = config.zones.keys().map(String::as_str).collect::<Vec<_>>().join(", ");
        return Err(format!("unknown zone {zone:?}: the config names {names}"));
    }
    Ok(zone.clone())
}
