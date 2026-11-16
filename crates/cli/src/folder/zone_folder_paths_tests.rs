//! zone_folder_paths: None for a zone the config does not name. The paths it gives run request_folder_paths, another
//! file, so they are proved by the ZIT crates/cli/tests/zone_folder_paths_tests.rs (S-102).

use std::collections::BTreeMap;

use super::zone_folder_paths;
use crate::config::{Config, ZoneConfig};

fn config() -> Config {
    Config {
        origin_branch: "main".to_string(),
        integration_branch: "integration".to_string(),
        remote: "origin".to_string(),
        zones: BTreeMap::from([
            ("core".to_string(), ZoneConfig { paths: vec!["src/core".to_string(), "Cargo.toml".to_string()] }),
            ("web".to_string(), ZoneConfig { paths: vec!["web".to_string()] }),
        ]),
        root_files: vec!["Cargo.toml".to_string(), "Makefile".to_string()],
        build_dirs: Vec::new(),
        cap_bytes: 1,
        floor_bytes: 0,
        formatter: None,
    }
}

#[test]
fn a_zone_not_in_the_config_has_none() {
    assert_eq!(zone_folder_paths(&config(), "docs"), None);
}
