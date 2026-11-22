//! parse_squash_args: one configured zone, with at most one --message text before or after it, is the answer; no
//! word, two words, an unknown zone, and a --message with no text, an empty or blank text, or given twice are
//! refused.

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
fn the_zone_and_the_message_are_read() {
    for (line, zone, message) in [
        (&["b"][..], "b", None),
        (&["b", "--message", "Subject\n\nBody."], "b", Some("Subject\n\nBody.")),
        (&["--message", " leading space kept ", "a"], "a", Some(" leading space kept ")),
        (&["a", "--message=x: y"], "a", Some("x: y")),
        (&["--message=--json", "b"], "b", Some("--json")),
    ] {
        let expected = Ok((zone.to_string(), message.map(str::to_string)));
        assert_eq!(parse_squash_args(&words(line), &config()), expected, "{line:?}");
    }
}

#[test]
fn bad_lines_are_refused_with_the_reason() {
    for (line, reason) in [
        (&[][..], "squash needs a zone"),
        (&["a", "b"], "unexpected word \"b\" after the zone"),
        (&["--force"], "unknown zone \"--force\": the config names a, b"),
        (&["a", "--message"], "--message needs a text"),
        (&["--message", "", "a"], "--message needs a text that is not empty or only whitespace"),
        (&["a", "--message="], "--message needs a text that is not empty or only whitespace"),
        (&["a", "--message", " \n\t"], "--message needs a text that is not empty or only whitespace"),
        (&["a", "--message", "one", "--message=two"], "--message is given twice"),
        (&["--message", "text"], "squash needs a zone"),
        (&["a", "b", "--message"], "unexpected word \"b\" after the zone"),
    ] {
        assert_eq!(parse_squash_args(&words(line), &config()), Err(reason.to_string()), "{line:?}");
    }
}
