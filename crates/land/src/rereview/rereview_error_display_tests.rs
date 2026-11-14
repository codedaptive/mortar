//! Refusal texts: each refusal names its commits, an absent change ID printing as `none`.
//! The Git and Trailers texts embed gitio's and oplog's own failure text, so they are left to Nagatha's
//! cross-zone tests.

use gitio::Oid;
use oplog::ChangeId;

use super::RereviewError;

fn oid(text: &str) -> Oid {
    Oid::parse(text).unwrap()
}

#[test]
fn not_same_change_names_both_commits_and_ids() {
    let err = RereviewError::NotSameChange {
        reviewed: oid("1111111111111111111111111111111111111111"),
        reviewed_id: Some(ChangeId::parse("kzlymxnwovpuqtrssrtqupvowxnymlzk").unwrap()),
        new: oid("2222222222222222222222222222222222222222"),
        new_id: None,
    };
    assert_eq!(
        err.to_string(),
        "not the same change: 1111111111111111111111111111111111111111 carries Mortar-Change-Id \
         kzlymxnwovpuqtrssrtqupvowxnymlzk, 2222222222222222222222222222222222222222 carries none"
    );
}

#[test]
fn not_same_change_without_reviewed_id_prints_none() {
    let err = RereviewError::NotSameChange {
        reviewed: oid("1111111111111111111111111111111111111111"),
        reviewed_id: None,
        new: oid("2222222222222222222222222222222222222222"),
        new_id: Some(ChangeId::parse("zzzzzzzzzzzzzzzzkkkkkkkkkkkkkkkk").unwrap()),
    };
    assert_eq!(
        err.to_string(),
        "not the same change: 1111111111111111111111111111111111111111 carries Mortar-Change-Id none, \
         2222222222222222222222222222222222222222 carries zzzzzzzzzzzzzzzzkkkkkkkkkkkkkkkk"
    );
}

#[test]
fn no_parent_names_the_commit() {
    let err = RereviewError::NoParent { commit: oid("1111111111111111111111111111111111111111") };
    assert_eq!(err.to_string(), "1111111111111111111111111111111111111111 has no parent to compare against");
}

#[test]
fn bases_differ_names_both_parents() {
    let err = RereviewError::BasesDiffer {
        reviewed_parent: oid("cccccccccccccccccccccccccccccccccccccccc"),
        new_parent: oid("dddddddddddddddddddddddddddddddddddddddd"),
    };
    assert_eq!(
        err.to_string(),
        "different parents: reviewed on cccccccccccccccccccccccccccccccccccccccc, new on \
         dddddddddddddddddddddddddddddddddddddddd; re-review across a moved origin is not supported yet"
    );
}
