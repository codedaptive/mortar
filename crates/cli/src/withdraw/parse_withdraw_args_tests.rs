//! parse_withdraw_args: one configured zone is the answer; no word, two words and an unknown zone are refused.

use std::collections::BTreeMap;

use super::parse_withdraw_args;
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

#[test]
fn each_line_is_its_zone_or_its_reason() {
    for (line, expected) in [
        (&["b"][..], Ok("b")),
        (&[][..], Err("withdraw needs a zone")),
        (&["a", "b"], Err("unexpected word \"b\" after the zone")),
        (&["c"], Err("unknown zone \"c\": the config names a, b")),
    ] {
        let words: Vec<String> = line.iter().map(|word| word.to_string()).collect();
        let expected = expected.map(str::to_string).map_err(str::to_string);
        assert_eq!(parse_withdraw_args(&words, &config()), expected, "{line:?}");
    }
}
