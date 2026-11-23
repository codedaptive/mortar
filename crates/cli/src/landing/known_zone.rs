//! The check that a landing verb's zone word is a zone the config names.

use crate::config::Config;

/// Ok when the config names `zone`; else the reason, "unknown zone "<zone>": the config names " and the config's
/// zones in name order joined by ", ".
pub(crate) fn known_zone(zone: &str, config: &Config) -> Result<(), String> {
    if config.zones.contains_key(zone) {
        return Ok(());
    }
    let names = config.zones.keys().map(String::as_str).collect::<Vec<_>>().join(", ");
    Err(format!("unknown zone {zone:?}: the config names {names}"))
}
