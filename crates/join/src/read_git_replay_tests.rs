use crate::read_git_replay::read_git_replay;
use crate::JoinError;
use gitio::{Oid, RefUpdate, ReplayOutcome};

const NEW: &str = "2222222222222222222222222222222222222222";
const OTHER_NEW: &str = "3333333333333333333333333333333333333333";
const OLD: &str = "4444444444444444444444444444444444444444";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn update(refname: &str, new: &str) -> RefUpdate {
    RefUpdate { refname: refname.to_string(), new: oid(new), old: Some(oid(OLD)) }
}

#[test]
fn the_tip_update_gives_its_new_value() {
    let outcome = ReplayOutcome::Replayed(vec![update("refs/heads/topic", NEW)]);
    assert_eq!(read_git_replay(outcome, "refs/heads/topic"), Ok(Some(oid(NEW))));
}

#[test]
fn the_tip_update_is_found_when_it_is_second() {
    let outcome =
        ReplayOutcome::Replayed(vec![update("refs/heads/other", OTHER_NEW), update("refs/heads/topic", NEW)]);
    assert_eq!(read_git_replay(outcome, "refs/heads/topic"), Ok(Some(oid(NEW))));
}

#[test]
fn no_updates_gives_none_so_the_fallback_runs() {
    let outcome = ReplayOutcome::Replayed(Vec::new());
    assert_eq!(read_git_replay(outcome, "refs/heads/topic"), Ok(None));
}

#[test]
fn updates_without_the_tip_are_refused_naming_the_tip() {
    let outcome = ReplayOutcome::Replayed(vec![update("refs/heads/other", OTHER_NEW)]);
    assert_eq!(
        read_git_replay(outcome, "refs/heads/topic"),
        Err(JoinError::TipNotReplayed("refs/heads/topic".to_string()))
    );
}

#[test]
fn conflicted_gives_none() {
    assert_eq!(read_git_replay(ReplayOutcome::Conflicted, "refs/heads/topic"), Ok(None));
}
