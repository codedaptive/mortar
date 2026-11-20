//! encode_config: one line, keys in config order, zones in name order, formatter null or its object.

use std::collections::BTreeMap;

use super::{encode_config, Config, FormatterConfig, ZoneConfig};

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn bare() -> Config {
    Config {
        origin_branch: "main".to_string(),
        integration_branch: "integration".to_string(),
        remote: "origin".to_string(),
        zones: BTreeMap::new(),
        root_files: Vec::new(),
        build_dirs: strings(&["target"]),
        cap_bytes: 104857600,
        floor_bytes: 0,
        formatter: None,
    }
}

#[test]
fn a_config_without_zones_or_formatter() {
    assert_eq!(
        encode_config(&bare()),
        "{\"origin_branch\":\"main\",\"integration_branch\":\"integration\",\"remote\":\"origin\",\"zones\":{},\
         \"root_files\":[],\"build_dirs\":[\"target\"],\"cap_bytes\":104857600,\"floor_bytes\":0,\
         \"formatter\":null}\n"
    );
}

#[test]
fn zones_in_name_order_and_the_formatter_object() {
    let mut config = bare();
    config.zones.insert("web".to_string(), ZoneConfig { paths: strings(&["apps/web", "libs/ui"]) });
    config.zones.insert("api".to_string(), ZoneConfig { paths: strings(&["svc"]) });
    config.root_files = strings(&["Cargo.lock"]);
    config.formatter = Some(FormatterConfig { command: strings(&["cargo", "fmt"]), timeout_seconds: 30 });
    assert_eq!(
        encode_config(&config),
        "{\"origin_branch\":\"main\",\"integration_branch\":\"integration\",\"remote\":\"origin\",\
         \"zones\":{\"api\":{\"paths\":[\"svc\"]},\"web\":{\"paths\":[\"apps/web\",\"libs/ui\"]}},\
         \"root_files\":[\"Cargo.lock\"],\"build_dirs\":[\"target\"],\"cap_bytes\":104857600,\"floor_bytes\":0,\
         \"formatter\":{\"command\":[\"cargo\",\"fmt\"],\"timeout_seconds\":30}}\n"
    );
}

#[test]
fn strings_are_escaped_and_stay_on_one_line() {
    let mut config = bare();
    config.remote = "a\"b\nc".to_string();
    let text = encode_config(&config);
    assert!(text.contains(r#""remote":"a\"b\nc""#), "{text}");
    assert_eq!(text.matches('\n').count(), 1);
    assert!(text.ends_with("}\n"));
}
