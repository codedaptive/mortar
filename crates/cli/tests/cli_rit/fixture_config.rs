//! Input words for binary init in validation. The binary owns config encoding.
use std::collections::BTreeMap;
use super::{FormatterConfig, ZoneConfig};

#[derive(Clone)]
pub struct FixtureConfig {
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
