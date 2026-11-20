//! Mortar's settings for one repository.
//! A bare type; decode_config_tests, encode_config_tests and protected_refs_tests exercise it.

use std::collections::BTreeMap;

use super::{FormatterConfig, ZoneConfig};

/// The three branch and remote names are non-empty; Mortar holds no default names. `zones` is keyed
/// by zone name. `cap_bytes` is the snapshot size cap, `floor_bytes` the free-disk floor; `formatter` None
/// means the team runs none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub origin_branch: String,
    pub integration_branch: String,
    pub remote: String,
    pub zones: BTreeMap<String, ZoneConfig>,
    pub root_files: Vec<String>,
    pub build_dirs: Vec<String>,
    pub cap_bytes: u64,
    pub floor_bytes: u64,
    pub formatter: Option<FormatterConfig>,
}
