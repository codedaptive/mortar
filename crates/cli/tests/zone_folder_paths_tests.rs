//! zone_folder_paths: a zone's paths, then the root files, a root file the zone already lists kept at its first
//! place only.
//! ZIT (rule 11, S-102): zone_folder_paths runs folder/request_folder_paths for real, inside the cli zone; it calls no
//! crate below cli.

use std::collections::BTreeMap;

use cli::{zone_folder_paths, Config, ZoneConfig};

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
fn a_zone_has_its_paths_then_the_root_files() {
    assert_eq!(
        zone_folder_paths(&config(), "web"),
        Some(vec!["web".to_string(), "Cargo.toml".to_string(), "Makefile".to_string()])
    );
}

#[test]
fn a_root_file_the_zone_already_lists_is_not_repeated() {
    assert_eq!(
        zone_folder_paths(&config(), "core"),
        Some(vec!["src/core".to_string(), "Cargo.toml".to_string(), "Makefile".to_string()])
    );
}
