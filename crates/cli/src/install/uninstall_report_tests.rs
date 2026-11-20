//! uninstall_report: "not-installed" naming the record when there was none, else "uninstalled" with a line per item;
//! exit 0 both ways, the items as a --json member.

use std::path::{Path, PathBuf};

use crate::report::{ExitCode, OutValue, Report};

use super::{uninstall_report, InstallItem};

#[test]
fn no_record_is_not_installed_and_says_where_it_looked() {
    assert_eq!(
        uninstall_report(Path::new("/h/.local/state/mortar/install.json"), None),
        Report {
            verb: "uninstall".to_string(),
            exit: ExitCode::Done,
            outcome: "not-installed".to_string(),
            lines: vec!["nothing to remove: no install record at /h/.local/state/mortar/install.json".to_string()],
            owner: None,
            next: "mortar install".to_string(),
            fields: vec![("items".to_string(), OutValue::Raw(vec![]))],
        }
    );
}

#[test]
fn a_record_is_uninstalled_with_a_line_and_an_object_per_item() {
    let items = [
        InstallItem { action: "removed".to_string(), path: PathBuf::from("/h/.local/bin/mortar"), note: String::new() },
        InstallItem {
            action: "left".to_string(),
            path: PathBuf::from("/h/.claude/skills/mortar/SKILL.md"),
            note: "changed since install".to_string(),
        },
    ];
    assert_eq!(
        uninstall_report(Path::new("/h/.local/state/mortar/install.json"), Some(&items)),
        Report {
            verb: "uninstall".to_string(),
            exit: ExitCode::Done,
            outcome: "uninstalled".to_string(),
            lines: vec![
                "removed /h/.local/bin/mortar".to_string(),
                "left /h/.claude/skills/mortar/SKILL.md: changed since install".to_string(),
            ],
            owner: None,
            next: "mortar install, to put Mortar back".to_string(),
            fields: vec![(
                "items".to_string(),
                OutValue::Raw(vec![
                    "{\"action\":\"removed\",\"path\":\"/h/.local/bin/mortar\",\"note\":\"\"}".to_string(),
                    "{\"action\":\"left\",\"path\":\"/h/.claude/skills/mortar/SKILL.md\",\"note\":\"changed since install\"}"
                        .to_string(),
                ]),
            )],
        }
    );
}
