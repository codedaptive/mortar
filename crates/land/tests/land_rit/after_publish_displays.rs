//! WO-land-rit-3: AfterPublishError::fmt -> actual Gitio, Oplog and Join formatters (S-101).
//! Each expectation is literal protocol text, not the nested formatter's answer.
use gitio::{GitError, Oid};
use join::JoinError;
use land::{AfterPublishError, LandError, SquashError};
use oplog::OplogError;
use testkit::TestRepo;

fn a() -> Oid { Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap() }
fn b() -> Oid { Oid::parse("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap() }
fn git_error() -> GitError {
    GitError { args: vec!["read-tree".into(), "HEAD".into()], code: Some(128), stderr: "fatal: no tree\n".into() }
}
macro_rules! case {
    ($name:ident, $value:expr, $expected:literal) => {
        #[test] fn $name() { let _repo = TestRepo::new().unwrap(); assert_eq!(($value).to_string(), $expected); }
    };
}
case!(rit_after_publish_git_error_keeps_the_git_diagnostic, AfterPublishError::Git(git_error()),
    "after-publish git: git read-tree HEAD: exit code 128: fatal: no tree");
case!(rit_after_publish_squash_error_runs_the_nested_git_formatter, AfterPublishError::Squash(SquashError::Git(git_error())),
    "after-publish: git read-tree HEAD: exit code 128: fatal: no tree");
case!(rit_after_publish_join_error_runs_the_join_formatter, AfterPublishError::Join(JoinError::MergeCommit(a())),
    "after-publish replay: commit aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa is a merge commit; replay takes one-parent commits only");
case!(rit_after_publish_land_error_runs_the_nested_oplog_formatter,
    AfterPublishError::Land(LandError::Oplog(OplogError::UnknownOp { op_id: "op-404".into() })),
    "after-publish: landing op log: cannot restore: op op-404 is not in the op log; find the op id with mortar log");
case!(rit_after_publish_unrelated_names_zone_tip_then_origin, AfterPublishError::Unrelated { zone: "core".into(), origin: a(), tip: b() },
    "zone core at bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb shares no history with origin aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa; the zone was not moved");
case!(rit_after_publish_missing_source_names_the_original_oid, AfterPublishError::SquashOfNotInZone { zone: "core".into(), squash_of: a() },
    "the published squash of zone core was made from aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa, which zone core does not hold; the zone was not moved");
