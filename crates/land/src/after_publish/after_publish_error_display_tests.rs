//! Display for AfterPublishError: the NoSquashOf, Unrelated and SquashOfNotInZone refusals as literal text, and the
//! Git, Squash, Join and Land arms with the nested gitio, oplog and join texts they embed.

use super::AfterPublishError;
use crate::landing::LandError;
use crate::squash::SquashError;
use gitio::{GitError, Oid};
use join::JoinError;
use oplog::OplogError;

fn a() -> Oid { Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap() }
fn b() -> Oid { Oid::parse("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap() }
fn git_error() -> GitError {
    GitError { args: vec!["read-tree".into(), "HEAD".into()], code: Some(128), stderr: "fatal: no tree\n".into() }
}

#[test]
fn no_squash_of_names_the_zone_and_says_it_was_not_moved() {
    let error = AfterPublishError::NoSquashOf { zone: "land".to_string() };
    assert_eq!(
        error.to_string(),
        "no squash of zone land on origin carries a Mortar-Squash-Of trailer; the zone was not moved"
    );
}

#[test]
fn after_publish_git_error_keeps_the_git_diagnostic() {
    assert_eq!(AfterPublishError::Git(git_error()).to_string(), "after-publish git: git read-tree HEAD: exit code 128: fatal: no tree");
}

#[test]
fn after_publish_squash_error_runs_the_nested_git_formatter() {
    assert_eq!(AfterPublishError::Squash(SquashError::Git(git_error())).to_string(), "after-publish: git read-tree HEAD: exit code 128: fatal: no tree");
}

#[test]
fn after_publish_join_error_runs_the_join_formatter() {
    assert_eq!(AfterPublishError::Join(JoinError::MergeCommit(a())).to_string(), "after-publish replay: commit aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa is a merge commit; replay takes one-parent commits only");
}

#[test]
fn after_publish_land_error_runs_the_nested_oplog_formatter() {
    assert_eq!(AfterPublishError::Land(LandError::Oplog(OplogError::UnknownOp { op_id: "op-404".into() })).to_string(), "after-publish: landing op log: cannot restore: op op-404 is not in the op log; find the op id with mr log");
}

#[test]
fn after_publish_unrelated_names_zone_tip_then_origin() {
    assert_eq!(AfterPublishError::Unrelated { zone: "core".into(), origin: a(), tip: b() }.to_string(), "zone core at bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb shares no history with origin aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa; the zone was not moved");
}

#[test]
fn after_publish_missing_source_names_the_original_oid() {
    assert_eq!(AfterPublishError::SquashOfNotInZone { zone: "core".into(), squash_of: a() }.to_string(), "the published squash of zone core was made from aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, which zone core does not hold; the zone was not moved");
}
