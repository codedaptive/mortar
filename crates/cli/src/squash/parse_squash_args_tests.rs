//! parse_squash_args: one configured zone is the answer; no word, two words and an unknown zone are refused.

use std::collections::BTreeMap;

use super::parse_squash_args;
use crate::config::{Config, ZoneConfig};

fn config() -> Config {
    Config {
        origin_branch: "main".to_string(),
        integration_branch: "integration".to_string(),
        remote: "origin".to_string(),
        zones: BTreeMap::from([
            ("a".to_string(), ZoneConfig { paths: vec!["src/a".to_string()] }),
            ("b".to_string(), ZoneConfig { paths: vec!["src/b".to_string()] }),
        ]),
        root_files: Vec::new(),
        build_dirs: Vec::new(),
        cap_bytes: 1,
        floor_bytes: 0,
        formatter: None,
    }
}

fn words(list: &[&str]) -> Vec<String> {
    list.iter().map(|word| word.to_string()).collect()
}

#[test]
fn a_configured_zone_is_the_zone() {
    assert_eq!(parse_squash_args(&words(&["b"]), &config()), Ok("b".to_string()));
}

#[test]
fn no_word_is_refused() {
    assert_eq!(parse_squash_args(&[], &config()), Err("squash needs a zone".to_string()));
}

#[test]
fn a_second_word_is_refused_naming_it() {
    assert_eq!(
        parse_squash_args(&words(&["a", "b"]), &config()),
        Err("unexpected word \"b\" after the zone".to_string())
    );
}

#[test]
fn an_unknown_zone_is_refused_naming_it_and_the_configs_zones() {
    assert_eq!(
        parse_squash_args(&words(&["c"]), &config()),
        Err("unknown zone \"c\": the config names a, b".to_string())
    );
}
