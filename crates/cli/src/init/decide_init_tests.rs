//! decide_init: no config writes; equal is already done, with or without --replace; different replaces or is
//! refused, naming the differing keys in config order.

use std::collections::BTreeMap;

use super::{decide_init, InitDecision};
use crate::config::{Config, FormatterConfig, ZoneConfig};

fn config() -> Config {
    Config {
        origin_branch: "main".to_string(),
        integration_branch: "integration".to_string(),
        remote: "origin".to_string(),
        zones: BTreeMap::from([("cli".to_string(), ZoneConfig { paths: vec!["crates/cli".to_string()] })]),
        root_files: vec!["Cargo.toml".to_string()],
        build_dirs: vec!["target".to_string()],
        cap_bytes: 100,
        floor_bytes: 200,
        formatter: None,
    }
}

fn keys(items: &[&str]) -> Vec<String> {
    items.iter().map(|k| k.to_string()).collect()
}

#[test]
fn no_config_is_write() {
    assert_eq!(decide_init(None, &config(), false), InitDecision::Write);
    assert_eq!(decide_init(None, &config(), true), InitDecision::Write);
}

#[test]
fn the_same_values_are_already_done_with_or_without_replace() {
    assert_eq!(decide_init(Some(&config()), &config(), false), InitDecision::AlreadyDone);
    assert_eq!(decide_init(Some(&config()), &config(), true), InitDecision::AlreadyDone);
}

#[test]
fn other_values_are_refused_naming_the_key() {
    let wanted = Config { remote: "upstream".to_string(), ..config() };
    assert_eq!(decide_init(Some(&config()), &wanted, false), InitDecision::Refused { keys: keys(&["remote"]) });
}

#[test]
fn other_values_with_replace_are_replaced_naming_the_key() {
    let wanted = Config { cap_bytes: 101, ..config() };
    assert_eq!(decide_init(Some(&config()), &wanted, true), InitDecision::Replace { keys: keys(&["cap_bytes"]) });
}

#[test]
fn every_key_differing_is_named_in_config_order() {
    let wanted = Config {
        origin_branch: "trunk".to_string(),
        integration_branch: "int".to_string(),
        remote: "up".to_string(),
        zones: BTreeMap::new(),
        root_files: Vec::new(),
        build_dirs: vec![".build".to_string()],
        cap_bytes: 1,
        floor_bytes: 2,
        formatter: Some(FormatterConfig { command: vec!["fmt".to_string()], timeout_seconds: 9 }),
    };
    assert_eq!(
        decide_init(Some(&config()), &wanted, false),
        InitDecision::Refused {
            keys: keys(&[
                "origin_branch", "integration_branch", "remote", "zones", "root_files", "build_dirs", "cap_bytes",
                "floor_bytes", "formatter",
            ])
        }
    );
}

#[test]
fn a_zone_path_differing_names_zones() {
    let mut wanted = config();
    wanted.zones.insert("cli".to_string(), ZoneConfig { paths: vec!["crates/cli".to_string(), "x".to_string()] });
    wanted.floor_bytes = 7;
    assert_eq!(
        decide_init(Some(&config()), &wanted, true),
        InitDecision::Replace { keys: keys(&["zones", "floor_bytes"]) }
    );
}
