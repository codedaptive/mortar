//! known_zone: a zone the config names is Ok; any other word is refused, named, with the config's zones in name order.

use std::collections::BTreeMap;

use super::known_zone;
use crate::config::{Config, ZoneConfig};

fn config() -> Config {
    Config {
        origin_branch: "main".to_string(),
        integration_branch: "integration".to_string(),
        remote: "origin".to_string(),
        zones: BTreeMap::from([
            ("b".to_string(), ZoneConfig { paths: vec!["src/b".to_string()] }),
            ("a".to_string(), ZoneConfig { paths: vec!["src/a".to_string()] }),
        ]),
        root_files: Vec::new(),
        build_dirs: Vec::new(),
        cap_bytes: 1,
        floor_bytes: 0,
        formatter: None,
    }
}

#[test]
fn a_named_zone_is_ok_and_any_other_word_is_its_reason() {
    for (zone, expected) in [
        ("a", Ok(())),
        ("b", Ok(())),
        ("c", Err("unknown zone \"c\": the config names a, b")),
        ("A", Err("unknown zone \"A\": the config names a, b")),
    ] {
        assert_eq!(known_zone(zone, &config()), expected.map_err(str::to_string), "{zone:?}");
    }
}
