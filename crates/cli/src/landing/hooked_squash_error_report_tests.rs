//! hooked_squash_error_report: each HookedSquashError arm's report.

use gitio::{HookRefusal, Oid};
use land::{HookedSquashError, SquashError};

use super::{hooked_squash_error_report, squash_error_report};
use crate::report::{ExitCode, OutValue, Report};

const A: &str = "1111111111111111111111111111111111111111";

fn refused(lines: &[&str], owner: &str, next: &str, fields: Vec<(String, OutValue)>) -> Report {
    Report {
        verb: "squash".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: lines.iter().map(|line| line.to_string()).collect(),
        owner: Some(owner.to_string()),
        next: next.to_string(),
        fields,
    }
}

#[test]
fn each_hooked_squash_error_has_its_report() {
    let squash = || SquashError::AttrSourceMissing { commit: Oid::parse(A).unwrap() };
    let rows = [
        ("Squash", HookedSquashError::Squash(squash()), squash_error_report("squash", &squash())),
        (
            "Refused",
            HookedSquashError::Refused(HookRefusal {
                hook: "pre-commit",
                code: Some(7),
                stdout: b"out\n".to_vec(),
                stderr: b"err\n".to_vec(),
            }),
            refused(
                &[
                    "the pre-commit hook refused the commit, exit 7",
                    "no squash commit was written and no ref moved",
                    "stdout: out",
                    "stderr: err",
                ],
                "whoever maintains the pre-commit hook",
                "fix what the pre-commit hook reports, then mr squash again",
                vec![
                    ("hook".to_string(), OutValue::Text("pre-commit".to_string())),
                    ("code".to_string(), OutValue::Number(7)),
                    ("stdout".to_string(), OutValue::Text("out\n".to_string())),
                    ("stderr".to_string(), OutValue::Text("err\n".to_string())),
                ],
            ),
        ),
        (
            "Unreadable",
            HookedSquashError::Unreadable {
                part: "Mortar-Zone".to_string(),
                message: "a long hooked message".to_string(),
            },
            refused(
                &[
                    "the commit hooks left a squash message whose Mortar-Zone does not read back",
                    "no squash commit was written and no ref moved",
                ],
                "whoever maintains the commit-msg hook",
                "repeat mr squash; if it fails again, the commit message part named is unreadable",
                Vec::new(),
            ),
        ),
    ];
    for (name, error, expected) in rows {
        assert_eq!(hooked_squash_error_report("squash", &error), expected, "{name}");
    }
}
