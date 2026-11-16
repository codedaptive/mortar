//! rereview_error_report: NotSameChange (with and without change ids) and NoParent in the cli's own words. The Git
//! and Trailers cases are RITs (see the file).

use gitio::Oid;
use land::RereviewError;
use oplog::ChangeId;

use super::rereview_error_report;
use crate::report::{ExitCode, OutValue, Report};

const REVIEWED: &str = "1111111111111111111111111111111111111111";
const NEW: &str = "2222222222222222222222222222222222222222";
const NEXT: &str = "name two squash commits of one change: mortar rereview <reviewed commit> <new commit> --actor <name>";

fn oid(text: &str) -> Oid {
    Oid::parse(text).unwrap()
}

fn change(text: &str) -> Option<ChangeId> {
    Some(ChangeId::parse(text).unwrap())
}

fn not_same_change(reviewed_id: Option<ChangeId>, new_id: Option<ChangeId>) -> Report {
    rereview_error_report(&RereviewError::NotSameChange { reviewed: oid(REVIEWED), reviewed_id, new: oid(NEW), new_id })
}

fn refused_not_same(line: &str) -> Report {
    Report {
        verb: "rereview".to_string(),
        exit: ExitCode::Refused,
        outcome: "not-same-change".to_string(),
        lines: vec![line.to_string()],
        owner: Some("the two squash commits".to_string()),
        next: NEXT.to_string(),
        fields: vec![
            ("reviewed".to_string(), OutValue::Text("1111111111111111111111111111111111111111".to_string())),
            ("new".to_string(), OutValue::Text("2222222222222222222222222222222222222222".to_string())),
        ],
    }
}

#[test]
fn two_change_ids_are_both_named() {
    assert_eq!(
        not_same_change(change("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk"), change("zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz")),
        refused_not_same(
            "1111111111111111111111111111111111111111 carries change kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk and \
             2222222222222222222222222222222222222222 carries change zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz; they are not \
             one change"
        )
    );
}

#[test]
fn a_commit_without_a_change_id_says_so() {
    assert_eq!(
        not_same_change(None, change("lmnopqrstuvwxyzklmnopqrstuvwxyzk")),
        refused_not_same(
            "1111111111111111111111111111111111111111 carries no Mortar-Change-Id and \
             2222222222222222222222222222222222222222 carries change lmnopqrstuvwxyzklmnopqrstuvwxyzk; they are not \
             one change"
        )
    );
    assert_eq!(
        not_same_change(change("lmnopqrstuvwxyzklmnopqrstuvwxyzk"), None),
        refused_not_same(
            "1111111111111111111111111111111111111111 carries change lmnopqrstuvwxyzklmnopqrstuvwxyzk and \
             2222222222222222222222222222222222222222 carries no Mortar-Change-Id; they are not one change"
        )
    );
    assert_eq!(
        not_same_change(None, None),
        refused_not_same(
            "1111111111111111111111111111111111111111 carries no Mortar-Change-Id and \
             2222222222222222222222222222222222222222 carries no Mortar-Change-Id; they are not one change"
        )
    );
}

#[test]
fn a_root_commit_is_refused_naming_it() {
    assert_eq!(
        rereview_error_report(&RereviewError::NoParent { commit: oid(NEW) }),
        Report {
            verb: "rereview".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "2222222222222222222222222222222222222222 is a root commit, so it has no parent to diff from".to_string()
            ],
            owner: Some("the commit 2222222222222222222222222222222222222222".to_string()),
            next: NEXT.to_string(),
            fields: Vec::new(),
        }
    );
}
