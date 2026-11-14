//! parse_zone_flag: one zone with its paths in order; no `=`, no name, an empty path or a repeat refused.

use std::collections::BTreeMap;

use super::parse_zone_flag;
use crate::config::ZoneConfig;

fn paths(items: &[&str]) -> ZoneConfig {
    ZoneConfig { paths: items.iter().map(|p| p.to_string()).collect() }
}

#[test]
fn one_path() {
    let mut zones = BTreeMap::new();
    assert_eq!(parse_zone_flag(&mut zones, "orc=crates/cli"), Ok(()));
    assert_eq!(zones, BTreeMap::from([("orc".to_string(), paths(&["crates/cli"]))]));
}

#[test]
fn several_paths_keep_their_order() {
    let mut zones = BTreeMap::new();
    assert_eq!(parse_zone_flag(&mut zones, "orc=src,Cargo.toml,a/b"), Ok(()));
    assert_eq!(zones["orc"], paths(&["src", "Cargo.toml", "a/b"]));
}

#[test]
fn a_path_may_hold_an_equals_sign() {
    let mut zones = BTreeMap::new();
    assert_eq!(parse_zone_flag(&mut zones, "z=a=b"), Ok(()));
    assert_eq!(zones["z"], paths(&["a=b"]));
}

#[test]
fn no_equals_sign_is_refused() {
    let mut zones = BTreeMap::new();
    assert_eq!(parse_zone_flag(&mut zones, "orc"), Err("--zone needs <zone>=<path>[,<path>...], not \"orc\"".to_string()));
    assert!(zones.is_empty());
}

#[test]
fn no_zone_name_is_refused() {
    assert_eq!(parse_zone_flag(&mut BTreeMap::new(), "=src"), Err("--zone \"=src\" has no zone name".to_string()));
}

#[test]
fn an_empty_path_is_refused() {
    let mut zones = BTreeMap::new();
    assert_eq!(parse_zone_flag(&mut zones, "orc=src,"), Err("--zone \"orc\" has an empty path".to_string()));
    assert_eq!(parse_zone_flag(&mut zones, "orc="), Err("--zone \"orc\" has an empty path".to_string()));
    assert!(zones.is_empty());
}

#[test]
fn a_zone_given_twice_is_refused() {
    let mut zones = BTreeMap::new();
    assert_eq!(parse_zone_flag(&mut zones, "orc=src"), Ok(()));
    assert_eq!(parse_zone_flag(&mut zones, "orc=lib"), Err("--zone \"orc\" is given twice".to_string()));
    assert_eq!(zones["orc"], paths(&["src"]));
}
