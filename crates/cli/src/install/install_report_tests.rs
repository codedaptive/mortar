//! Tests of install_report: outcome by steps, a line and a JSON object per item, the plan's next (spec 07 item 8).

use std::path::PathBuf;

use super::{install_report, InstallItem, InstallPlan, InstallStep};
use crate::report::{ExitCode, OutValue};

fn plan(steps: Vec<InstallStep>) -> InstallPlan {
    InstallPlan {
        steps,
        items: vec![
            InstallItem {
                action: "placed".to_string(),
                path: PathBuf::from("/h/.local/bin/mortar"),
                note: String::new(),
            },
            InstallItem { action: "wiring skipped".to_string(), path: PathBuf::new(), note: "none picked".to_string() },
        ],
        next: "mr init, in the repository Mortar is to keep".to_string(),
        command: "mr",
    }
}

#[test]
fn steps_make_it_installed_with_a_line_and_object_per_item() {
    let step = InstallStep::Link { at: PathBuf::from("/h/.local/bin/mr"), target: PathBuf::from("/h/.local/bin/mortar") };
    let report = install_report(&plan(vec![step]));
    assert_eq!(report.verb, "install");
    assert_eq!(report.exit, ExitCode::Done);
    assert_eq!(report.outcome, "installed");
    assert_eq!(report.lines, ["placed /h/.local/bin/mortar", "wiring skipped: none picked"]);
    assert_eq!(report.owner, None);
    assert_eq!(report.next, "mr init, in the repository Mortar is to keep");
    let items = vec![
        r#"{"action":"placed","path":"/h/.local/bin/mortar","note":""}"#.to_string(),
        r#"{"action":"wiring skipped","path":"","note":"none picked"}"#.to_string(),
    ];
    assert_eq!(report.fields, [("items".to_string(), OutValue::Raw(items))]);
}

#[test]
fn no_step_is_already_installed() {
    let report = install_report(&plan(Vec::new()));
    assert_eq!(report.exit, ExitCode::Done);
    assert_eq!(report.outcome, "already-installed");
}
