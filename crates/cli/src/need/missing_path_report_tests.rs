//! missing_path_report: a path not at the folder's commit is exit 1, names the path and the commit, says nothing
//! ran, and is this request's to fix.

use gitio::Oid;

use super::missing_path_report;
use crate::report::{ExitCode, OutValue, Report};

const COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";

#[test]
fn a_missing_path_is_refused_naming_the_path_and_the_commit() {
    let commit = Oid::parse(COMMIT).unwrap();
    assert_eq!(
        missing_path_report("kit/nowhere", &commit),
        Report {
            verb: "need".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "kit/nowhere is not in the repository at the folder's commit 0123456789abcdef0123456789abcdef01234567"
                    .to_string(),
                "nothing ran".to_string(),
            ],
            owner: Some("this request".to_string()),
            next: "mortar need <a path the repository holds>".to_string(),
            fields: vec![
                ("path".to_string(), OutValue::Text("kit/nowhere".to_string())),
                ("commit".to_string(), OutValue::Text("0123456789abcdef0123456789abcdef01234567".to_string())),
            ],
        },
    );
}
