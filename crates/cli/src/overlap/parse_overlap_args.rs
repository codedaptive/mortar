//! `mr overlap <zone>`'s words: exactly one, a zone the config names.

use crate::config::Config;

/// The zone named in `rest`. Refused, first match wins: any `--` on the line; a word starting with `-`, the first
/// in line order; no word; a word after the zone; a zone that is not a key of `config`'s zones, naming those keys
/// (or saying the config names none).
pub fn parse_overlap_args(rest: &[String], dash_dash: Option<usize>, config: &Config) -> Result<String, String> {
    if dash_dash.is_some() {
        return Err("overlap takes no `--`".to_string());
    }
    if let Some(dashed) = rest.iter().find(|word| word.starts_with('-')) {
        return Err(format!("unexpected word {dashed:?}"));
    }
    let [zone, after @ ..] = rest else {
        return Err("overlap needs a zone".to_string());
    };
    if let Some(extra) = after.first() {
        return Err(format!("unexpected word {extra:?} after the zone"));
    }
    if !config.zones.contains_key(zone) {
        let known = config.zones.keys().map(String::as_str).collect::<Vec<_>>().join(", ");
        if known.is_empty() {
            return Err(format!("unknown zone {zone:?}: the config names no zone"));
        }
        return Err(format!("unknown zone {zone:?}: the config names {known}"));
    }
    Ok(zone.clone())
}
