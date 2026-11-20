//! parse_overlap_args: exactly one word, a zone the config names; no word, a second word, a dashed word, an unknown
//! zone and any `--` are refused.

use std::collections::BTreeMap;

use super::parse_overlap_args;
use crate::config::{Config, ZoneConfig};

fn config() -> Config {
    Config {
        origin_branch: "main".to_string(),
        integration_branch: "integration".to_string(),
        remote: "origin".to_string(),
        zones: BTreeMap::from([
            ("cli".to_string(), ZoneConfig { paths: vec!["crates/cli".to_string()] }),
            ("join".to_string(), ZoneConfig { paths: vec!["crates/join".to_string()] }),
            ("land".to_string(), ZoneConfig { paths: vec!["crates/land".to_string()] }),
        ]),
        root_files: Vec::new(),
        build_dirs: Vec::new(),
        cap_bytes: 1,
        floor_bytes: 0,
        formatter: None,
    }
}

fn parse(rest: &[&str], dash_dash: Option<usize>) -> Result<String, String> {
    parse_overlap_args(&rest.iter().map(ToString::to_string).collect::<Vec<_>>(), dash_dash, &config())
}

#[test]
fn one_zone_the_config_names_is_read() {
    assert_eq!(parse(&["join"], None), Ok("join".to_string()));
    assert_eq!(parse(&["cli"], None), Ok("cli".to_string()));
}

#[test]
fn no_word_is_refused() {
    assert_eq!(parse(&[], None), Err("overlap needs a zone".to_string()));
}

#[test]
fn a_second_word_is_refused_by_name() {
    assert_eq!(parse(&["cli", "land"], None), Err("unexpected word \"land\" after the zone".to_string()));
}

#[test]
fn a_dashed_word_is_refused_by_name() {
    assert_eq!(parse(&["cli", "--all"], None), Err("unexpected word \"--all\"".to_string()));
}

#[test]
fn an_unknown_zone_is_refused_naming_the_configs_zones() {
    assert_eq!(parse(&["guard"], None), Err("unknown zone \"guard\": the config names cli, join, land".to_string()));
}

#[test]
fn a_double_dash_is_refused_before_anything_else() {
    assert_eq!(parse(&["cli"], Some(1)), Err("overlap takes no `--`".to_string()));
    assert_eq!(parse(&[], Some(0)), Err("overlap takes no `--`".to_string()));
    assert_eq!(parse(&["guard", "land"], Some(0)), Err("overlap takes no `--`".to_string()));
}

#[test]
fn a_config_with_no_zones_says_so() {
    let empty = Config { zones: BTreeMap::new(), ..config() };
    assert_eq!(
        parse_overlap_args(&["cli".to_string()], None, &empty),
        Err("unknown zone \"cli\": the config names no zone".to_string())
    );
}
