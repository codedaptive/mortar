//! Display for RereviewError's refusals: NotSameChange names both commits and their Mortar-Change-Ids (none for a
//! commit without one), NoParent names the commit; Git and Trailers carry gitio's and oplog's own text.

use gitio::{GitError, Oid};
use oplog::{ChangeId, MortarTrailerError};

use super::RereviewError;

fn a() -> Oid { Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap() }
fn b() -> Oid { Oid::parse("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap() }
fn id() -> ChangeId { ChangeId::parse("kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk").unwrap() }
fn git_error() -> GitError {
    GitError { args: vec!["read-tree".into(), "HEAD".into()], code: Some(128), stderr: "fatal: no tree\n".into() }
}

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

#[test]
fn rereview_git_display_adds_its_prefix_to_the_real_gitio_error_text() {
    assert_eq!(RereviewError::Git(git_error()).to_string(), "git failed: git read-tree HEAD: exit code 128: fatal: no tree");
}

#[test]
fn rereview_trailers_display_is_the_real_nested_oplog_gitio_text_without_a_land_prefix() {
    assert_eq!(RereviewError::Trailers(MortarTrailerError::Git(git_error())).to_string(),
        "git trailers failed: git read-tree HEAD: exit code 128: fatal: no tree");
}
