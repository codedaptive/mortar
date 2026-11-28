//! Owed op/message seams through PUBLIC squash/land_batch/publish, real Git objects and Oplog records.
//! Single-member and multi-member back-out metadata are checked together in batch_ops.rs.
//! Normal completed push only: interrupted remote-push outcomes are tested in remote_intents.rs.
use crate::support::{Fixture, CHANGE_ID};
use gitio::{Oid, RunOpts};
use land::{BatchLandAnswer, LandRefs, PublishAnswer, Squash, SquashAnswer};
use oplog::{KillPidProbe, LockName, OpPhase, OpRecord, OpResult, RefMove};
use std::{fmt::Debug, time::{SystemTime, UNIX_EPOCH}};

const ACTOR: &str = "land-rit-2";
const SQUASH_REF: &str = "refs/mortar/squash/core";
const INTEGRATION: &str = "refs/heads/integration";
pub(super) fn refs() -> LandRefs { LandRefs { origin_branch: "main".into(), integration_branch: "integration".into(), remote: "origin".into() } }
fn now() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() }
pub(super) fn locked<T, E: Debug>(f: &Fixture, work: impl FnOnce() -> Result<T, E>) -> T {
    oplog::with_lock(&f.git, &f.common, &LockName::Land, "rit-ops-lock", &KillPidProbe, || {
        oplog::recover(&f.git, &f.common).unwrap(); work()
    }).unwrap()
}
pub(super) fn raw(f: &Fixture, args: &[&str]) { f.git.run(args, &RunOpts::default()).unwrap(); }
fn records(f: &Fixture, kind: &str) -> Vec<OpRecord> {
    oplog::read_op_log(&f.common).unwrap().records.into_iter().filter(|r| r.kind == kind).collect()
}
fn made(f: &Fixture) -> Squash {
    let answer = locked(f, || land::squash(&f.git, &refs(), "core", ACTOR));
    let SquashAnswer::Made(squash) = answer else { panic!("fresh fixture must make a squash: {answer:?}") };
    squash
}
fn landed(f: &Fixture, squash: &Squash) -> Oid {
    let answer = locked(f, || land::land_batch(&f.git, &refs(), squash, ACTOR));
    let BatchLandAnswer::Landed { integration } = answer else { panic!("fixture must land: {answer:?}") };
    integration
}

// Expected templates below are the literal protocol contract at the call sites, never the private builder.
// Op IDs are nonempty runtime identities; per-attempt keys correlate with them. The publish-push key
// is instead stable and remote-qualified, with no op ID, so a repeat can find the marked intent.
// Timestamp is checked against independent clock reads surrounding the call, not against a speed limit.
fn assert_op(f: &Fixture, kind: &str, key_prefix: &str, reason: &str, move_: RefMove, before: u64, after: u64) {
    let read = records(f, kind); assert_eq!(read.len(), 2); assert!(!read[0].op_id.is_empty());
    assert_eq!(read[0].op_id, read[1].op_id);
    let key = if kind == "publish-push" { key_prefix.to_string() } else { format!("{key_prefix}:{}", read[0].op_id) };
    for (record, (phase, result)) in read.iter().zip([(OpPhase::Intent, None), (OpPhase::Done, Some(OpResult::Moved))]) {
        assert!((before..=after).contains(&record.at));
        assert_eq!(*record, OpRecord {
            op_id: read[0].op_id.clone(), key: key.clone(), kind: kind.into(),
            actor: ACTOR.into(), reason: reason.into(), at: record.at, phase, moves: vec![move_.clone()], result,
        });
    }
}

#[test]
fn rit_a_remade_squash_moves_the_ref_from_the_prior_commit() {
    let f = Fixture::new(); let old = f.git.read_ref(SQUASH_REF).unwrap().unwrap();
    let before = now(); let squash = made(&f); let after = now();
    assert_eq!(squash.change_id.as_str(), CHANGE_ID);
    assert_op(&f, "land-squash", &format!("land-squash:refs/mortar/squash/core:{old}:{}", squash.commit),
        &format!("squash moves refs/mortar/squash/core from {old} to {}", squash.commit),
        RefMove { refname: SQUASH_REF.into(), old: Some(old), new: squash.commit.clone() }, before, after);
    assert_eq!(f.git.read_ref(SQUASH_REF).unwrap(), Some(squash.commit));
}

#[test]
fn rit_a_first_squash_creates_the_ref() {
    let f = Fixture::new(); raw(&f, &["update-ref", "-d", SQUASH_REF]);
    let before = now(); let squash = made(&f); let after = now();
    assert_op(&f, "land-squash", &format!("land-squash:refs/mortar/squash/core:none:{}", squash.commit),
        &format!("squash moves refs/mortar/squash/core from nothing to {}", squash.commit),
        RefMove { refname: SQUASH_REF.into(), old: None, new: squash.commit.clone() }, before, after);
    assert_eq!(f.git.read_ref(SQUASH_REF).unwrap(), Some(squash.commit));
}

#[test]
fn rit_squash_identical_move_inputs_make_different_attempt_keys() {
    let f = Fixture::new(); let old = f.git.read_ref(SQUASH_REF).unwrap().unwrap();
    let first = made(&f);
    // Restore only this disposable fixture's prior ref, so both public attempts have identical move inputs.
    raw(&f, &["update-ref", SQUASH_REF, old.as_str()]);
    let second = made(&f); assert_eq!(first.commit, second.commit);
    let read = records(&f, "land-squash"); assert_eq!(read.len(), 4);
    assert_eq!(read[0].moves, read[2].moves); assert_eq!(read[0].reason, read[2].reason);
    assert_ne!(read[0].op_id, read[2].op_id); assert_ne!(read[0].key, read[2].key);
    for record in [&read[0], &read[2]] {
        assert_eq!(record.key, format!("land-squash:refs/mortar/squash/core:{old}:{}:{}", first.commit, record.op_id));
    }
}

#[test]
fn rit_a_landing_moves_integration_from_origin_to_the_landed_commit() {
    let f = Fixture::new(); let base = f.git.read_ref("refs/heads/main").unwrap().unwrap(); let squash = made(&f);
    let before = now(); let new = landed(&f, &squash); let after = now(); assert_eq!(new, squash.commit);
    assert_op(&f, "land", &format!("land:refs/heads/integration:{base}:{new}"),
        &format!("land kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk moves refs/heads/integration from {base} to {new}"),
        RefMove { refname: INTEGRATION.into(), old: Some(base), new: new.clone() }, before, after);
    assert_eq!(f.git.read_ref(INTEGRATION).unwrap(), Some(new));
}

#[test]
fn rit_integration_identical_move_inputs_make_different_attempt_keys() {
    let f = Fixture::new(); let base = f.git.read_ref(INTEGRATION).unwrap().unwrap(); let squash = made(&f);
    let first = landed(&f, &squash);
    raw(&f, &["update-ref", INTEGRATION, base.as_str()]);
    assert_eq!(landed(&f, &squash), first);
    let read = records(&f, "land"); assert_eq!(read.len(), 4);
    assert_eq!(read[0].moves, read[2].moves); assert_eq!(read[0].reason, read[2].reason);
    assert_ne!(read[0].op_id, read[2].op_id); assert_ne!(read[0].key, read[2].key);
    for record in [&read[0], &read[2]] {
        assert_eq!(record.key, format!("land:refs/heads/integration:{base}:{first}:{}", record.op_id));
    }
}

#[test]
fn rit_the_push_intent_moves_the_origin_branch_from_expected_to_new() {
    let f = Fixture::new(); let base = f.git.read_ref("refs/heads/main").unwrap().unwrap();
    let remote = f.git.repo.join("rit-remote.git");
    raw(&f, &["init", "--bare", "--quiet", remote.to_str().unwrap()]);
    raw(&f, &["remote", "add", "origin", remote.to_str().unwrap()]);
    raw(&f, &["push", "--quiet", "origin", &format!("{base}:refs/heads/main")]);
    let squash = made(&f); let new = landed(&f, &squash); let before = now();
    let answer = locked(&f, || land::publish(&f.git, &refs(), ACTOR)); let after = now();
    assert_eq!(answer, PublishAnswer::Published { origin: new.clone() });
    assert_op(&f, "publish-push", &format!("publish-push:origin:refs/heads/main:{base}:{new}"),
        &format!("publish-push kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk moves refs/heads/main from {base} to {new}"),
        RefMove { refname: "refs/heads/main".into(), old: Some(base), new: new.clone() }, before, after);
    let push_records = oplog::read_marked_log(&f.common).unwrap().records.into_iter()
        .filter(|record| record.record.kind == "publish-push").collect::<Vec<_>>();
    assert_eq!(push_records.len(), 2);
    for record in push_records { assert_eq!(record.remote.as_deref(), Some("origin")); }
    assert_eq!(f.git.read_ref("refs/heads/main").unwrap(), Some(new.clone()));
    let remote_git = gitio::Git { repo: remote, ..f.git.clone() };
    assert_eq!(remote_git.read_ref("refs/heads/main").unwrap(), Some(new));
}

#[test]
fn rit_landing_message_names_the_change_id_and_zone_with_a_trailing_newline() {
    let f = Fixture::new(); let base = f.git.read_ref("refs/heads/main").unwrap().unwrap(); let squash = made(&f);
    let tree = f.git.commit_info(&base).unwrap().tree;
    // An independent origin commit forces the merge-landing path that emits landing_message.
    let origin = f.git.commit_tree(&tree, &[base], "independent origin commit\n", None, None).unwrap();
    raw(&f, &["update-ref", "refs/heads/main", origin.as_str()]); raw(&f, &["update-ref", INTEGRATION, origin.as_str()]);
    let new = landed(&f, &squash); let info = f.git.commit_info(&new).unwrap();
    assert_eq!(info.parents, vec![origin, squash.commit]);
    assert_eq!(info.message, "land kkkkkkkkkkkkkkkkkkkkkkkkkkkkkkkk (core)\n");
}
