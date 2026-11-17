//! parse_after_publish_args: the published zone and each --move, and each word it refuses, the first failure in the
//! contract's order winning: `--` first, then the words left to right, then a missing zone, then each zone in order.

use std::collections::BTreeMap;

use super::parse_after_publish_args;
use crate::after_publish::AfterPublishArgs;
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

fn parse(rest: &[&str], dash_dash: Option<usize>) -> Result<AfterPublishArgs, String> {
    parse_after_publish_args(&rest.iter().map(ToString::to_string).collect::<Vec<_>>(), dash_dash, &config())
}

fn refused(rest: &[&str]) -> String {
    match parse(rest, None) {
        Err(reason) => reason,
        Ok(args) => panic!("{rest:?} gave {args:?}"),
    }
}

fn args(published: &str, moves: &[&str]) -> AfterPublishArgs {
    AfterPublishArgs { published: published.to_string(), moves: moves.iter().map(ToString::to_string).collect() }
}

#[test]
fn the_published_zone_alone_has_no_moves() {
    assert_eq!(parse(&["cli"], None), Ok(args("cli", &[])));
}

#[test]
fn each_move_is_read_in_order_wherever_it_stands() {
    assert_eq!(parse(&["cli", "--move", "land", "--move", "join"], None), Ok(args("cli", &["land", "join"])));
    assert_eq!(parse(&["--move", "join", "cli"], None), Ok(args("cli", &["join"])));
    assert_eq!(parse(&["--move", "land", "cli", "--move", "join"], None), Ok(args("cli", &["land", "join"])));
}

#[test]
fn a_double_dash_is_refused_before_anything_else() {
    assert_eq!(parse(&["cli"], Some(1)), Err("after-publish takes no `--`".to_string()));
    assert_eq!(parse(&[], Some(0)), Err("after-publish takes no `--`".to_string()));
    assert_eq!(parse(&["nowhere", "-x"], Some(0)), Err("after-publish takes no `--`".to_string()));
}

#[test]
fn move_without_a_value_is_refused() {
    assert_eq!(refused(&["cli", "--move"]), "--move needs a zone");
    assert_eq!(refused(&["--move"]), "--move needs a zone");
}

#[test]
fn a_dashed_word_is_refused_by_name() {
    assert_eq!(refused(&["cli", "--move=land"]), "unexpected word \"--move=land\"");
    assert_eq!(refused(&["--verbose", "cli"]), "unexpected word \"--verbose\"");
    assert_eq!(refused(&["cli", "-m", "land"]), "unexpected word \"-m\"");
}

#[test]
fn a_second_plain_word_is_refused_by_name() {
    assert_eq!(refused(&["cli", "land"]), "unexpected word \"land\" after the zone");
    assert_eq!(refused(&["cli", "--move", "join", "land"]), "unexpected word \"land\" after the zone");
}

#[test]
fn the_first_bad_word_from_the_left_wins() {
    assert_eq!(refused(&["cli", "land", "--verbose"]), "unexpected word \"land\" after the zone");
    assert_eq!(refused(&["cli", "--verbose", "land"]), "unexpected word \"--verbose\"");
    assert_eq!(refused(&["cli", "-x", "--move"]), "unexpected word \"-x\"");
    assert_eq!(refused(&["cli", "land", "--move"]), "unexpected word \"land\" after the zone");
}

#[test]
fn no_published_zone_is_refused() {
    let reason = "after-publish needs the zone whose squash was published";
    assert_eq!(refused(&[]), reason);
    assert_eq!(refused(&["--move", "land"]), reason);
}

#[test]
fn a_word_failure_wins_over_no_published_zone() {
    assert_eq!(refused(&["--move", "land", "--move"]), "--move needs a zone");
    assert_eq!(refused(&["-q"]), "unexpected word \"-q\"");
}

#[test]
fn an_unknown_zone_is_refused_naming_it_and_the_configs_zones() {
    assert_eq!(refused(&["guard"]), "unknown zone \"guard\": the config names cli, join, land");
    assert_eq!(refused(&["cli", "--move", "guard"]), "unknown zone \"guard\": the config names cli, join, land");
}

#[test]
fn a_word_failure_wins_over_an_unknown_zone() {
    assert_eq!(refused(&["guard", "oplog"]), "unexpected word \"oplog\" after the zone");
    assert_eq!(refused(&["guard", "--move"]), "--move needs a zone");
}

#[test]
fn a_zone_named_twice_is_refused() {
    assert_eq!(refused(&["cli", "--move", "cli"]), "zone \"cli\" is named twice");
    assert_eq!(refused(&["cli", "--move", "land", "--move", "land"]), "zone \"land\" is named twice");
}

#[test]
fn zones_are_checked_published_first_then_each_move_in_order() {
    assert_eq!(refused(&["guard", "--move", "oplog"]), "unknown zone \"guard\": the config names cli, join, land");
    assert_eq!(
        refused(&["cli", "--move", "oplog", "--move", "cli"]),
        "unknown zone \"oplog\": the config names cli, join, land"
    );
    assert_eq!(refused(&["cli", "--move", "cli", "--move", "oplog"]), "zone \"cli\" is named twice");
    assert_eq!(
        refused(&["cli", "--move", "land", "--move", "land", "--move", "oplog"]),
        "zone \"land\" is named twice"
    );
}
