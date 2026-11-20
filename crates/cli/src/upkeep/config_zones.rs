//! The zone list disk and sweep pass to folders: every key of the config's zones, never an empty list.

use crate::config::Config;

/// Every key of `config.zones` in map order; None when the map is empty. Given an empty zone list, folders picks
/// every zone's replay records (refs/mortar/replayed/) for deletion (inside folders), so an empty list is never
/// handed out: the caller refuses with no_zones_report instead.
pub fn config_zones(config: &Config) -> Option<Vec<String>> {
    if config.zones.is_empty() {
        return None;
    }
    Some(config.zones.keys().cloned().collect())
}
