//! with_restored: an undo's report gains the paused join put back (or already put back), its conflicted files and
//! their --json fields; exit, outcome, next and owner stay as they were.

use std::path::PathBuf;

use ::join::{PausedJoin, RestoredJoin};
use gitio::Oid;

use super::with_restored;
use crate::report::{ExitCode, OutValue, Report};

const COMMIT: &str = "dddddddddddddddddddddddddddddddddddddddd";

fn base() -> Report {
    Report {
        verb: "undo".to_string(),
        exit: ExitCode::Done,
        outcome: "undone".to_string(),
        lines: vec!["op op-7 was undone by op op-12".to_string()],
        owner: Some("orc/WO-cli-4".to_string()),
        next: "mr status".to_string(),
        fields: vec![("undone_op".to_string(), OutValue::Text("op-7".to_string()))],
    }
}

fn paused(files: &[&str]) -> PausedJoin {
    PausedJoin {
        commit: Oid::parse(COMMIT).expect("a valid oid"),
        files: files.iter().map(PathBuf::from).collect(),
    }
}

fn state() -> PathBuf {
    PathBuf::from("/w/.git/mortar-join.json")
}

#[test]
fn a_restored_join_adds_its_commit_and_files() {
    let restored = RestoredJoin::Restored { state: state(), paused: paused(&["src/a.rs", "docs/b.md"]) };
    assert_eq!(
        with_restored(base(), &restored),
        Report {
            verb: "undo".to_string(),
            exit: ExitCode::Done,
            outcome: "undone".to_string(),
            lines: vec![
                "op op-7 was undone by op op-12".to_string(),
                "the paused join is put back: it stopped at commit dddddddddddddddddddddddddddddddddddddddd".to_string(),
                "conflicted files: src/a.rs, docs/b.md".to_string(),
            ],
            owner: Some("orc/WO-cli-4".to_string()),
            next: "mr status".to_string(),
            fields: vec![
                ("undone_op".to_string(), OutValue::Text("op-7".to_string())),
                ("paused_commit".to_string(), OutValue::Text(COMMIT.to_string())),
                ("conflicted".to_string(), OutValue::Texts(vec!["src/a.rs".to_string(), "docs/b.md".to_string()])),
            ],
        }
    );
}

#[test]
fn an_already_restored_join_says_so_and_omits_the_files_line_when_none() {
    let restored = RestoredJoin::AlreadyRestored { state: state(), paused: paused(&[]) };
    let report = with_restored(base(), &restored);
    assert_eq!(
        report.lines,
        vec![
            "op op-7 was undone by op op-12".to_string(),
            "the paused join was already put back: it stopped at commit dddddddddddddddddddddddddddddddddddddddd"
                .to_string(),
        ]
    );
    assert_eq!(
        report.fields,
        vec![
            ("undone_op".to_string(), OutValue::Text("op-7".to_string())),
            ("paused_commit".to_string(), OutValue::Text(COMMIT.to_string())),
            ("conflicted".to_string(), OutValue::Texts(Vec::new())),
        ]
    );
    assert_eq!(report.exit, ExitCode::Done);
    assert_eq!(report.outcome, "undone");
    assert_eq!(report.next, "mr status");
    assert_eq!(report.owner, Some("orc/WO-cli-4".to_string()));
}
