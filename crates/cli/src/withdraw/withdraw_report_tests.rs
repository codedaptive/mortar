//! withdraw_report: Withdrawn with and without a change ID, and Absent, each with its exit, outcome, lines, next and
//! fields.

use gitio::Oid;
use land::WithdrawAnswer;
use oplog::ChangeId;

use super::withdraw_report;
use crate::report::{ExitCode, OutValue, Report};

const COMMIT: &str = "1111111111111111111111111111111111111111";
const CHANGE: &str = "kkkkllllmmmmnnnnooooppppqqqqrrrr";
const NEW_SQUASH: &str =
    "mr squash core --actor <name> makes a new squash, with a new change ID, when the zone is ready for review";

fn done(outcome: &str, lines: &[&str], next: &str, withdrawn: OutValue, change_id: OutValue) -> Report {
    Report {
        verb: "withdraw".to_string(),
        exit: ExitCode::Done,
        outcome: outcome.to_string(),
        lines: lines.iter().map(|line| line.to_string()).collect(),
        owner: None,
        next: next.to_string(),
        fields: vec![
            ("zone".to_string(), OutValue::Text("core".to_string())),
            ("withdrawn".to_string(), withdrawn),
            ("change_id".to_string(), change_id),
        ],
    }
}

#[test]
fn each_answer_is_its_report() {
    let commit = Oid::parse(COMMIT).unwrap();
    let change_id = Some(ChangeId::parse(CHANGE).unwrap());
    let with_id = WithdrawAnswer::Withdrawn { commit: commit.clone(), change_id };
    let without_id = WithdrawAnswer::Withdrawn { commit, change_id: None };
    for (name, answer, expected) in [
        (
            "withdrawn with a change ID",
            with_id,
            done(
                "withdrawn",
                &[
                    "withdrew zone core's squash: commit 1111111111111111111111111111111111111111 (change \
                     kkkkllllmmmmnnnnooooppppqqqqrrrr)",
                    "refs/mortar/squash/core is deleted; no other ref moved",
                ],
                NEW_SQUASH,
                OutValue::Text(COMMIT.to_string()),
                OutValue::Text(CHANGE.to_string()),
            ),
        ),
        (
            "withdrawn without a change ID",
            without_id,
            done(
                "withdrawn",
                &[
                    "withdrew zone core's squash: commit 1111111111111111111111111111111111111111 (no change ID)",
                    "refs/mortar/squash/core is deleted; no other ref moved",
                ],
                NEW_SQUASH,
                OutValue::Text(COMMIT.to_string()),
                OutValue::Null,
            ),
        ),
        (
            "absent",
            WithdrawAnswer::Absent,
            done(
                "nothing-to-withdraw",
                &["zone core has no squash at refs/mortar/squash/core; nothing was written"],
                "nothing to do",
                OutValue::Null,
                OutValue::Null,
            ),
        ),
    ] {
        assert_eq!(withdraw_report("core", &answer), expected, "{name}");
    }
}
