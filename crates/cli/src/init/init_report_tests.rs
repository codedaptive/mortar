//! init_report: written, replaced and already-done are exit 0 with the folder step as next; refused is exit 1
//! naming the keys and owned by the existing config; every arm carries config_path.

use std::path::Path;

use super::{init_report, InitDecision};
use crate::report::{ExitCode, OutValue, Report};

const PATH: &str = "/r/.git/mortar/config.json";

fn path_field() -> Vec<(String, OutValue)> {
    vec![("config_path".to_string(), OutValue::Text(PATH.to_string()))]
}

#[test]
fn write_is_written() {
    assert_eq!(
        init_report(&InitDecision::Write, Path::new(PATH)),
        Report {
            verb: "init".to_string(),
            exit: ExitCode::Done,
            outcome: "written".to_string(),
            lines: vec!["wrote the Mortar config /r/.git/mortar/config.json".to_string()],
            owner: None,
            next: "mortar folder zone <zone> <path>".to_string(),
            fields: path_field(),
        }
    );
}

#[test]
fn replace_is_replaced_naming_the_keys() {
    let decision = InitDecision::Replace { keys: vec!["remote".to_string(), "zones".to_string()] };
    let report = init_report(&decision, Path::new(PATH));
    assert_eq!(report.exit as i32, 0);
    assert_eq!(report.outcome, "replaced");
    assert_eq!(report.lines, vec!["replaced the Mortar config /r/.git/mortar/config.json; changed: remote, zones".to_string()]);
    assert_eq!(report.owner, None);
    assert_eq!(report.next, "mortar folder zone <zone> <path>");
    assert_eq!(report.fields, path_field());
}

#[test]
fn already_done_changes_nothing() {
    let report = init_report(&InitDecision::AlreadyDone, Path::new(PATH));
    assert_eq!(report.exit, ExitCode::Done);
    assert_eq!(report.outcome, "already-done");
    assert_eq!(
        report.lines,
        vec![
            "nothing changed".to_string(),
            "the Mortar config /r/.git/mortar/config.json already holds these values".to_string(),
        ]
    );
    assert_eq!(report.next, "mortar folder zone <zone> <path>");
    assert_eq!(report.fields, path_field());
}

#[test]
fn refused_names_the_keys_and_the_owner() {
    let decision = InitDecision::Refused { keys: vec!["origin_branch".to_string(), "cap_bytes".to_string()] };
    assert_eq!(
        init_report(&decision, Path::new(PATH)),
        Report {
            verb: "init".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "the Mortar config /r/.git/mortar/config.json holds other values for: origin_branch, cap_bytes"
                    .to_string(),
                "nothing changed".to_string(),
            ],
            owner: Some("the existing config at /r/.git/mortar/config.json".to_string()),
            next: "mortar init with the same values, or add --replace".to_string(),
            fields: path_field(),
        }
    );
}
