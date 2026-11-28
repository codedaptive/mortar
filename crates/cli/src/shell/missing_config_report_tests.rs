//! missing_config_report: usage (exit 2) "no-config", naming the file, with `mr init` as next.

use std::path::Path;

use super::missing_config_report;
use crate::report::{ExitCode, Report};

#[test]
fn a_missing_config_is_usage_with_init_as_next() {
    assert_eq!(
        missing_config_report("join", Path::new("/r/.git/mortar/config.json")),
        Report {
            verb: "join".to_string(),
            exit: ExitCode::Usage,
            outcome: "no-config".to_string(),
            lines: vec![
                "there is no Mortar config at /r/.git/mortar/config.json".to_string(),
                "nothing ran".to_string(),
            ],
            owner: Some("whoever sets up Mortar in this repository".to_string()),
            next: "mr init".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn the_exit_code_is_2() {
    assert_eq!(missing_config_report("land", Path::new("/c")).exit as i32, 2);
}
