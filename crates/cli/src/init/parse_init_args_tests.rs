//! parse_init_args: the required names, each default, --zone forms and refusals, the formatter with and
//! without its timeout, repeats and unknown words.

use std::collections::BTreeMap;

use super::{parse_init_args, InitOptions};
use crate::config::{Config, FormatterConfig, ZoneConfig};

const REQUIRED: [&str; 6] = ["--origin", "main", "--integration", "integration", "--remote", "origin"];

fn words(extra: &[&str]) -> Vec<String> {
    REQUIRED.iter().chain(extra).map(|w| w.to_string()).collect()
}

fn ok(extra: &[&str]) -> InitOptions {
    match parse_init_args(&words(extra)) {
        Ok(options) => options,
        Err(reason) => panic!("{extra:?} gave {reason}"),
    }
}

fn err(all: &[&str]) -> String {
    let all = all.iter().map(|w| w.to_string()).collect::<Vec<_>>();
    match parse_init_args(&all) {
        Err(reason) => reason,
        Ok(options) => panic!("{all:?} gave {options:?}"),
    }
}

fn err_with(extra: &[&str]) -> String {
    err(&REQUIRED.iter().chain(extra).copied().collect::<Vec<_>>())
}

#[test]
fn the_three_names_alone_take_every_default() {
    assert_eq!(
        ok(&[]),
        InitOptions {
            config: Config {
                origin_branch: "main".to_string(),
                integration_branch: "integration".to_string(),
                remote: "origin".to_string(),
                zones: BTreeMap::new(),
                root_files: Vec::new(),
                build_dirs: vec!["target".to_string()],
                cap_bytes: 5_242_880,
                floor_bytes: 10_000_000_000,
                formatter: None,
            },
            replace: false,
        }
    );
}

#[test]
fn every_flag_given() {
    let options = ok(&[
        "--zone", "cli=crates/cli,Makefile", "--zone", "gitio=crates/gitio", "--root-file", "Cargo.toml",
        "--root-file", "Cargo.lock", "--build-dir", "target", "--build-dir", ".build", "--cap-bytes", "1024",
        "--floor-bytes", "0", "--formatter", "cargo", "--formatter", "fmt", "--formatter-timeout", "60", "--replace",
    ]);
    assert!(options.replace);
    let config = options.config;
    assert_eq!(
        config.zones,
        BTreeMap::from([
            ("gitio".to_string(), ZoneConfig { paths: vec!["crates/gitio".to_string()] }),
            ("cli".to_string(), ZoneConfig { paths: vec!["crates/cli".to_string(), "Makefile".to_string()] }),
        ])
    );
    assert_eq!(config.root_files, vec!["Cargo.toml".to_string(), "Cargo.lock".to_string()]);
    assert_eq!(config.build_dirs, vec!["target".to_string(), ".build".to_string()]);
    assert_eq!((config.cap_bytes, config.floor_bytes), (1024, 0));
    assert_eq!(
        config.formatter,
        Some(FormatterConfig { command: vec!["cargo".to_string(), "fmt".to_string()], timeout_seconds: 60 })
    );
}

#[test]
fn flags_may_come_in_any_order() {
    let options = parse_init_args(&["--remote", "up", "--replace", "--origin", "o", "--integration", "i"].map(String::from));
    let config = options.expect("parses").config;
    assert_eq!((config.origin_branch.as_str(), config.integration_branch.as_str(), config.remote.as_str()), ("o", "i", "up"));
}

#[test]
fn each_name_is_required() {
    assert_eq!(err(&["--integration", "i", "--remote", "r"]), "--origin is required");
    assert_eq!(err(&["--origin", "o", "--remote", "r"]), "--integration is required");
    assert_eq!(err(&["--origin", "o", "--integration", "i"]), "--remote is required");
}

#[test]
fn a_name_given_twice_is_refused() {
    assert_eq!(err_with(&["--origin", "trunk"]), "--origin is given twice");
    assert_eq!(err_with(&["--cap-bytes", "1", "--cap-bytes", "2"]), "--cap-bytes is given twice");
    assert_eq!(err_with(&["--replace", "--replace"]), "--replace is given twice");
}

#[test]
fn a_missing_or_empty_value_is_refused() {
    assert_eq!(err_with(&["--root-file"]), "--root-file needs a value");
    assert_eq!(err(&["--origin", "", "--integration", "i", "--remote", "r"]), "--origin needs a non-empty value");
}

#[test]
fn zone_refusals_come_through() {
    assert_eq!(err_with(&["--zone", "cli"]), "--zone needs <zone>=<path>[,<path>...], not \"cli\"");
    assert_eq!(err_with(&["--zone", "cli=a", "--zone", "cli=b"]), "--zone \"cli\" is given twice");
    assert_eq!(err_with(&["--zone", "cli=a,,b"]), "--zone \"cli\" has an empty path");
}

#[test]
fn numbers_are_whole_and_unsigned() {
    assert_eq!(err_with(&["--cap-bytes", "-1"]), "--cap-bytes needs a whole number, not \"-1\"");
    assert_eq!(err_with(&["--floor-bytes", "+5"]), "--floor-bytes needs a whole number, not \"+5\"");
    assert_eq!(err_with(&["--cap-bytes", "1.5"]), "--cap-bytes needs a whole number, not \"1.5\"");
    assert_eq!(
        err_with(&["--cap-bytes", "18446744073709551616"]),
        "--cap-bytes needs a whole number, not \"18446744073709551616\""
    );
    assert_eq!(ok(&["--cap-bytes", "18446744073709551615"]).config.cap_bytes, u64::MAX);
}

#[test]
fn a_formatter_needs_its_timeout() {
    assert_eq!(err_with(&["--formatter", "rustfmt"]), "--formatter needs --formatter-timeout <seconds>");
}

#[test]
fn a_timeout_needs_a_formatter() {
    assert_eq!(err_with(&["--formatter-timeout", "30"]), "--formatter-timeout needs --formatter");
}

#[test]
fn an_unknown_word_is_refused() {
    assert_eq!(err_with(&["--zones", "x"]), "unknown init argument \"--zones\"");
    assert_eq!(err_with(&["stray"]), "unknown init argument \"stray\"");
    assert_eq!(err(&["--origin=main"]), "unknown init argument \"--origin=main\"");
}
