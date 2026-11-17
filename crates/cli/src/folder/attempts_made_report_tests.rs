//! attempts_made_report: the attempt folders made under a directory, each with its base, and the next step.

use std::path::{Path, PathBuf};

use folders::{FolderKind, FolderRecord};
use gitio::Oid;

use super::attempts_made_report;
use crate::report::{ExitCode, OutValue, Report};

const TIP: &str = "cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd";

fn attempt(k: u32) -> (PathBuf, FolderRecord) {
    let record = FolderRecord {
        kind: FolderKind::Attempt(k),
        zone: "core".to_string(),
        request: Some("fix-1".to_string()),
        base: Oid::parse(TIP).expect("a valid oid"),
        work_order_paths: vec!["src/a".to_string()],
        needed_paths: Vec::new(),
        parent: Some("req/core/fix-1".to_string()),
    };
    (PathBuf::from(format!("/w/tries/try-{k}")), record)
}

#[test]
fn two_attempts_made() {
    assert_eq!(
        attempts_made_report(Path::new("/w/tries"), &[attempt(1), attempt(2)]),
        Report {
            verb: "folder".to_string(),
            exit: ExitCode::Done,
            outcome: "made".to_string(),
            lines: vec![
                "2 attempt folders are under /w/tries".to_string(),
                format!("attempt folder /w/tries/try-1 on base {TIP}"),
                format!("attempt folder /w/tries/try-2 on base {TIP}"),
            ],
            owner: None,
            next: "start one agent in each attempt folder; then mr folder keep <k> --actor <name> or mr \
                   folder drop --actor <name> in the request's folder"
                .to_string(),
            fields: vec![
                ("dir".to_string(), OutValue::Text("/w/tries".to_string())),
                (
                    "attempts".to_string(),
                    OutValue::Texts(vec!["/w/tries/try-1".to_string(), "/w/tries/try-2".to_string()]),
                ),
            ],
        }
    );
}
