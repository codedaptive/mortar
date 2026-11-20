//! One zone's entry in the config.
//! A bare type; decode_zones_tests and encode_config_tests exercise it.

/// The repository paths the zone owns, in the order given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoneConfig {
    pub paths: Vec<String>,
}
