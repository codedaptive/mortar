//! config_refusal: refused, the reason then that nothing changed, with fixing or removing the file as next.

use std::path::Path;

use super::config_refusal;
use crate::config::ConfigError;
use crate::report::{ExitCode, Report};

#[test]
fn an_unreadable_config_is_refused_naming_the_file() {
    let error = ConfigError { reason: "/r/.git/mortar/config.json is not a Mortar config: remote is missing".to_string() };
    assert_eq!(
        config_refusal("init", &error, Path::new("/r/.git/mortar/config.json")),
        Report {
            verb: "init".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "/r/.git/mortar/config.json is not a Mortar config: remote is missing".to_string(),
                "nothing changed".to_string(),
            ],
            owner: Some("whoever sets up Mortar in this repository".to_string()),
            next: "fix or remove /r/.git/mortar/config.json, then repeat the same command".to_string(),
            fields: Vec::new(),
        }
    );
}
