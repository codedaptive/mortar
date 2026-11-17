//! config_zones: every key of Config.zones in map order, never an empty list (S-121 (2)): an empty map gives None,
//! so no caller can pass folders an empty zone list, which would delete every zone's replay records.

use std::collections::BTreeMap;

use super::config_zones;
use crate::config::{Config, ZoneConfig};

fn config(zones: &[&str]) -> Config {
    Config {
        origin_branch: "main".to_string(),
        integration_branch: "integration".to_string(),
        remote: "origin".to_string(),
        zones: zones
            .iter()
            .map(|zone| (zone.to_string(), ZoneConfig { paths: vec![format!("crates/{zone}")] }))
            .collect::<BTreeMap<_, _>>(),
        root_files: Vec::new(),
        build_dirs: Vec::new(),
        cap_bytes: 1,
        floor_bytes: 0,
        formatter: None,
    }
}

#[test]
fn no_zones_gives_none_never_an_empty_list() {
    assert_eq!(config_zones(&config(&[])), None);
}

#[test]
fn one_zone_gives_it() {
    assert_eq!(config_zones(&config(&["cli"])), Some(vec!["cli".to_string()]));
}

#[test]
fn two_zones_give_both_in_map_order() {
    assert_eq!(config_zones(&config(&["land", "cli"])), Some(vec!["cli".to_string(), "land".to_string()]));
}

#[test]
fn every_answer_that_is_some_holds_at_least_one_zone() {
    for zones in [&["a"][..], &["a", "b"], &["z", "m", "a"]] {
        let listed = config_zones(&config(zones)).expect("a config with zones gives Some");
        assert_eq!(listed.len(), zones.len(), "{zones:?}");
    }
}
