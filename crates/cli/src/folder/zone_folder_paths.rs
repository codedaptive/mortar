//! The sparse paths of a zone folder, from the config.

use super::request_folder_paths;
use crate::config::Config;

/// None for a zone not in `config.zones`; else the zone's paths in order, then the root files, a path given more
/// than once kept at its first place only (request_folder_paths' rule).
pub fn zone_folder_paths(config: &Config, zone: &str) -> Option<Vec<String>> {
    config.zones.get(zone).map(|entry| request_folder_paths(&entry.paths, config))
}
