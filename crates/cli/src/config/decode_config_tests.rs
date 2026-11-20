//! decode_config: the inverse of encode_config over many configs, and one refusal per rule, by key.

use std::collections::BTreeMap;

use super::{decode_config, encode_config, Config, ConfigError, FormatterConfig, ZoneConfig};

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

pub(super) fn reason(text: &str) -> String {
    match decode_config(text) {
        Err(ConfigError { reason }) => reason,
        Ok(config) => panic!("{text:?} gave {config:?}"),
    }
}

/// The valid config text with `key`'s value text swapped for `value`.
pub(super) fn with(key: &str, value: &str) -> String {
    let mut members = vec![
        ("origin_branch", r#""main""#),
        ("integration_branch", r#""integration""#),
        ("remote", r#""origin""#),
        ("zones", r#"{"a":{"paths":["x"]}}"#),
        ("root_files", "[]"),
        ("build_dirs", r#"["target"]"#),
        ("cap_bytes", "10"),
        ("floor_bytes", "0"),
        ("formatter", "null"),
    ];
    for member in &mut members {
        if member.0 == key {
            member.1 = value;
        }
    }
    let body: Vec<String> = members.iter().map(|(k, v)| format!("\"{k}\":{v}")).collect();
    format!("{{{}}}", body.join(","))
}

fn configs() -> Vec<Config> {
    let base = Config {
        origin_branch: "main".to_string(),
        integration_branch: "integration".to_string(),
        remote: "origin".to_string(),
        zones: BTreeMap::new(),
        root_files: Vec::new(),
        build_dirs: Vec::new(),
        cap_bytes: 0,
        floor_bytes: 0,
        formatter: None,
    };
    let mut full = base.clone();
    full.origin_branch = "trunk/ü \"q\"".to_string();
    full.zones.insert("b".to_string(), ZoneConfig { paths: strings(&["b/1", "b/2"]) });
    full.zones.insert("a".to_string(), ZoneConfig { paths: Vec::new() });
    full.root_files = strings(&["Cargo.toml", "", "tab\there"]);
    full.build_dirs = strings(&["target", "build"]);
    full.cap_bytes = u64::MAX;
    full.floor_bytes = 5368709120;
    full.formatter = Some(FormatterConfig { command: strings(&["fmt"]), timeout_seconds: 0 });
    let mut other = base.clone();
    other.remote = "up\\stream".to_string();
    other.formatter = Some(FormatterConfig { command: strings(&["a", "-b", "c d"]), timeout_seconds: 600 });
    vec![base, full, other]
}

#[test]
fn round_trip_returns_each_config() {
    for config in configs() {
        assert_eq!(decode_config(&encode_config(&config)), Ok(config));
    }
}

#[test]
fn keys_in_any_order_with_whitespace() {
    let text = " {\n \"formatter\" : { \"timeout_seconds\" : 3 , \"command\" : [ \"f\" ] } , \"floor_bytes\":1,\
                \"cap_bytes\":2,\"build_dirs\":[],\"root_files\":[\"r\"],\"zones\":{\"z\":{\"paths\":[\"p\"]}},\
                \"remote\":\"o\",\"integration_branch\":\"i\",\"origin_branch\":\"m\"}\n";
    let config = decode_config(text).map_err(|e| e.reason);
    let mut zones = BTreeMap::new();
    zones.insert("z".to_string(), ZoneConfig { paths: strings(&["p"]) });
    assert_eq!(
        config,
        Ok(Config {
            origin_branch: "m".to_string(),
            integration_branch: "i".to_string(),
            remote: "o".to_string(),
            zones,
            root_files: strings(&["r"]),
            build_dirs: Vec::new(),
            cap_bytes: 2,
            floor_bytes: 1,
            formatter: Some(FormatterConfig { command: strings(&["f"]), timeout_seconds: 3 }),
        })
    );
}

#[test]
fn the_valid_text_of_the_refusal_tests_decodes() {
    assert!(decode_config(&with("", "")).is_ok());
}

#[test]
fn top_level_shape_refusals() {
    assert_eq!(reason("[]"), "the config is not an object");
    assert_eq!(reason(r#"{"remote":"o","remote":"p"}"#), "key \"remote\" appears twice in the config");
    assert_eq!(reason(&with("remote", r#""o","extra":1"#)), "unknown key \"extra\" in the config");
    assert_eq!(reason(r#"{"origin_branch":"m"}"#), "missing key \"integration_branch\"");
    assert_eq!(reason(&with("formatter", "null}")), "unexpected '}' at byte 195 after the value");
    assert_eq!(reason(r#"{"origin_branch":"main","zones":{"a":{"paths":["x""#), "the text ends before the config is complete");
    assert_eq!(reason(&with("cap_bytes", "true")), "cap_bytes is not a number");
}

#[test]
fn name_refusals() {
    assert_eq!(reason(&with("origin_branch", r#""""#)), "origin_branch is empty; it must name something");
    assert_eq!(reason(&with("integration_branch", "null")), "integration_branch is not a string");
    assert_eq!(reason(&with("remote", "[]")), "remote is not a string");
}

#[test]
fn zone_refusals() {
    assert_eq!(reason(&with("zones", "[]")), "zones is not an object");
    assert_eq!(reason(&with("zones", r#"{"a":{"paths":[]},"a":{"paths":[]}}"#)), "zone \"a\" appears twice in zones");
    assert_eq!(reason(&with("zones", r#"{"a":["x"]}"#)), "zones.a is not an object");
    assert_eq!(reason(&with("zones", r#"{"a":{"paths":[],"owner":"x"}}"#)), "unknown key \"owner\" in zones.a");
    assert_eq!(reason(&with("zones", r#"{"a":{}}"#)), "missing key \"zones.a.paths\"");
    assert_eq!(reason(&with("zones", r#"{"a":{"paths":"x"}}"#)), "zones.a.paths is not an array of strings");
    assert_eq!(reason(&with("zones", r#"{"a":{"paths":["x",1]}}"#)), "zones.a.paths item 1 is not a string");
}

#[test]
fn list_and_number_refusals() {
    assert_eq!(reason(&with("root_files", r#""Cargo.toml""#)), "root_files is not an array of strings");
    assert_eq!(reason(&with("build_dirs", "[null]")), "build_dirs item 0 is not a string");
    assert_eq!(reason(&with("cap_bytes", r#""10""#)), "cap_bytes is not a number");
    assert_eq!(reason(&with("floor_bytes", "null")), "floor_bytes is not a number");
}

#[test]
fn formatter_refusals() {
    assert_eq!(reason(&with("formatter", r#"["f"]"#)), "formatter is neither null nor an object");
    assert_eq!(reason(&with("formatter", r#"{"command":[],"timeout_seconds":1}"#)), "formatter.command is empty; it must name the formatter's program");
    assert_eq!(reason(&with("formatter", r#"{"command":["f"]}"#)), "missing key \"formatter.timeout_seconds\"");
    assert_eq!(reason(&with("formatter", r#"{"command":["f"],"timeout_seconds":1,"env":[]}"#)), "unknown key \"env\" in formatter");
    assert_eq!(reason(&with("formatter", r#"{"command":"f","timeout_seconds":1}"#)), "formatter.command is not an array of strings");
}
