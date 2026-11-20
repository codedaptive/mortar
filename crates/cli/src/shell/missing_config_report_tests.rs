//! missing_config_report: refused "no-config", naming the file, with `mr init` as next.

use std::path::Path;

use super::missing_config_report;
use crate::report::{ExitCode, Report};

#[test]
fn a_missing_config_is_refused_with_init_as_next() {
    assert_eq!(
        missing_config_report("join", Path::new("/r/.git/mortar/config.json")),
        Report {
            verb: "join".to_string(),
            exit: ExitCode::Refused,
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
fn the_refusal_code_is_1() {
    assert_eq!(missing_config_report("land", Path::new("/c")).exit as i32, 1);
}
