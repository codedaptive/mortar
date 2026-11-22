//! The words of `mr withdraw` after its globals, read into the zone whose squash it withdraws.

use crate::config::Config;
use crate::landing::known_zone;

/// Exactly one word, a zone the config names. The Err is the reason: no word, a second word (named), or
/// known_zone's reason for a word that is not one of the config's zones.
pub fn parse_withdraw_args(rest: &[String], config: &Config) -> Result<String, String> {
    let Some((zone, words)) = rest.split_first() else {
        return Err("withdraw needs a zone".to_string());
    };
    if let Some(extra) = words.first() {
        return Err(format!("unexpected word {extra:?} after the zone"));
    }
    known_zone(zone, config)?;
    Ok(zone.clone())
}
