//! partner_conflict_report: a conflicted partner merge is refused exit 3, naming the partner and each file, owned by
//! both zones' coordinators.
//! Unit test: the report is pure.

use std::path::PathBuf;

use super::partner_conflict_report;
use crate::report::{ExitCode, OutValue, Report};

#[test]
fn a_conflicted_partner_merge_is_refused_naming_the_partner_and_its_files() {
    let files = [PathBuf::from("Cargo.toml"), PathBuf::from("web/w.txt")];
    let expected = Report {
        verb: "folder".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![
            "the merge of zone core with partner web conflicts on these files:".to_string(),
            "  Cargo.toml".to_string(),
            "  web/w.txt".to_string(),
            "nothing was made".to_string(),
        ],
        owner: Some("the coordinators of zones core and web".to_string()),
        next: "cut the request without --with web, or repeat once zones core and web agree on these files".to_string(),
        fields: vec![
            ("partner".to_string(), OutValue::Text("web".to_string())),
            ("files".to_string(), OutValue::Texts(vec!["Cargo.toml".to_string(), "web/w.txt".to_string()])),
        ],
    };
    assert_eq!(partner_conflict_report("core", "web", &files), expected);
}
