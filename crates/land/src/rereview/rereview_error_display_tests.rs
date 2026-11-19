//! Display for RereviewError's refusals: NotSameChange names both commits and their Mortar-Change-Ids (none for a
//! commit without one), NoParent names the commit.

use gitio::Oid;
use oplog::ChangeId;

use super::RereviewError;

fn a() -> Oid { Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap() }
fn b() -> Oid { Oid::parse("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap() }
fn id() -> ChangeId { ChangeId::parse("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk").unwrap() }

#[test]
fn not_same_change_names_both_commits_and_ids() {
    assert_eq!(RereviewError::NotSameChange { reviewed: a(), reviewed_id: Some(id()), new: b(), new_id: None }.to_string(), "not the same change: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa carries Mortar-Change-Id kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk, bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb carries none");
}

#[test]
fn not_same_change_without_reviewed_id_prints_none() {
    assert_eq!(RereviewError::NotSameChange { reviewed: a(), reviewed_id: None, new: b(), new_id: Some(id()) }.to_string(), "not the same change: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa carries Mortar-Change-Id none, bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb carries kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk");
}

#[test]
fn no_parent_names_the_commit() {
    assert_eq!(RereviewError::NoParent { commit: a() }.to_string(), "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa has no parent to compare against");
}
