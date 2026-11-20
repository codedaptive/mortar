//! Publish-repeat RITs: actual public Land publish killed around its real push -> Oplog recovery -> public repeat.
//! Parent fixtures own all repos; independently read remote refs, exact marked records and actual push trace decide.
use crate::{owed_ops::{locked, raw, refs}, push_fixture::{with_lock, Prepared, ACTOR, MAIN, SQUASH},
    push_kill::{interrupt, recover_leaves_open}, support::CHANGE_ID};
use gitio::Oid;
use land::{LandAnswer, PublishAnswer, PublishError};
use oplog::{MarkedRecord, OpPhase, OpRecord, OpResult, RefMove};
use std::time::{SystemTime, UNIX_EPOCH};

fn assert_record(p: &Prepared, m: &MarkedRecord, id: &str, phase: OpPhase, result: Option<OpResult>) {
    let r = &m.record; assert!(!id.is_empty());
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    assert!(r.at > 0 && r.at <= now, "timestamp is real metadata, not a duration oracle");
    assert_eq!(*m, MarkedRecord { record: OpRecord {
        op_id: id.into(), key: format!("publish-push:origin:{MAIN}:{}:{}", p.base, p.new),
        kind: "publish-push".into(), actor: ACTOR.into(),
        reason: format!("publish-push {CHANGE_ID} moves {MAIN} from {} to {}", p.base, p.new),
        at: r.at, phase, moves: vec![RefMove { refname: MAIN.into(), old: Some(p.base.clone()), new: p.new.clone() }], result,
    }, values: vec![], remote: Some("origin".into()) });
}
fn assert_open(p: &Prepared, remote: &Oid) -> String {
    let records = p.pushes(); assert_eq!(records.len(), 1);
    let id = records[0].record.op_id.clone(); assert_record(p, &records[0], &id, OpPhase::Intent, None);
    assert_eq!(p.remote_tip(), Some(remote.clone()));
    assert_eq!(p.f.git.read_ref(MAIN).unwrap(), Some(p.base.clone()));
    assert_eq!(p.f.git.read_ref("refs/heads/integration").unwrap(), Some(p.new.clone()));
    assert!(p.state().is_file()); assert_eq!(p.f.git.read_ref(SQUASH).unwrap(), Some(p.new.clone()));
    recover_leaves_open(p); assert_eq!(p.pushes(), records); id
}
fn failed(actual: Option<&Oid>) -> OpResult { OpResult::NotMoved { refs: vec![(MAIN.into(), actual.cloned())] } }
fn assert_closed(p: &Prepared, id: &str, phase: OpPhase, result: OpResult) {
    let records = p.pushes(); assert_eq!(records.len(), 2);
    assert_record(p, &records[0], id, OpPhase::Intent, None);
    assert_record(p, &records[1], id, phase, Some(result));
    assert!(records[1].record.at >= records[0].record.at);
}

#[test]
fn rit_remote_intent_killed_before_push_is_failed_then_a_fresh_attempt_pushes_once() {
    let p = Prepared::new(); let git = interrupt(&p, "before"); let id = assert_open(&p, &p.base);
    assert_eq!(p.publish(&git), PublishAnswer::Published { origin: p.new.clone() });
    let records = p.pushes(); assert_eq!(records.len(), 4); let next = &records[2].record.op_id;
    assert_ne!(next, &id); assert_record(&p, &records[0], &id, OpPhase::Intent, None);
    assert_record(&p, &records[1], &id, OpPhase::Failed, Some(failed(Some(&p.base))));
    assert_record(&p, &records[2], next, OpPhase::Intent, None);
    assert_record(&p, &records[3], next, OpPhase::Done, Some(OpResult::Moved));
    assert_eq!(p.trace_bytes(), b"push\n"); p.assert_finished(&p.new); p.assert_inert_repeat(&git);
}

#[test]
fn rit_remote_intent_killed_after_push_is_closed_done_once_without_another_push() {
    let p = Prepared::new(); let git = interrupt(&p, "after"); let id = assert_open(&p, &p.new);
    assert_eq!(p.publish(&git), PublishAnswer::Published { origin: p.new.clone() });
    assert_closed(&p, &id, OpPhase::Done, OpResult::Moved);
    assert_eq!(p.trace_bytes(), b"push\n"); p.assert_finished(&p.new); p.assert_inert_repeat(&git);
}

#[test]
fn rit_remote_intent_past_new_is_done_but_local_origin_stops_at_the_landing() {
    let p = Prepared::new(); let git = interrupt(&p, "after"); let id = assert_open(&p, &p.new);
    let past = p.foreign_push_on(&p.new);
    assert_eq!(p.remote.commit_info(&past).unwrap().parents, vec![p.new.clone()]);
    recover_leaves_open(&p); // Local main is still old; even here recovery must not mistake it for a failed push.
    assert_eq!(p.publish(&git), PublishAnswer::Published { origin: p.new.clone() });
    assert_closed(&p, &id, OpPhase::Done, OpResult::Moved);
    assert_eq!(p.trace_bytes(), b"push\n"); p.assert_finished(&past); p.assert_inert_repeat(&git);
}

#[test]
fn rit_remote_intent_remote_elsewhere_is_failed_then_the_same_squash_can_reland() {
    let p = Prepared::new(); let git = interrupt(&p, "before"); let id = assert_open(&p, &p.base);
    let elsewhere = p.foreign_push_on(&p.base);
    assert!(!p.f.git.is_ancestor(&p.new, &elsewhere).unwrap());
    recover_leaves_open(&p);
    assert_eq!(p.publish(&git), PublishAnswer::OriginMoved { found: elsewhere.clone() });
    assert_closed(&p, &id, OpPhase::Failed, failed(Some(&elsewhere)));
    assert_eq!(p.trace_bytes(), b""); assert_eq!(p.remote_tip(), Some(elsewhere.clone()));
    for branch in [MAIN, "refs/heads/integration"] { assert_eq!(p.f.git.read_ref(branch).unwrap(), Some(elsewhere.clone())); }
    assert!(!p.state().exists()); assert_eq!(p.f.git.read_ref(SQUASH).unwrap(), Some(p.squash.commit.clone()));
    p.assert_inert_repeat(&git);
    let LandAnswer::Landed { integration } = locked(&p.f, || land::land(&p.f.git, &refs(), &p.squash, ACTOR)) else {
        panic!("the same approved squash must land again");
    };
    assert_eq!(p.f.git.commit_info(&integration).unwrap().parents, vec![elsewhere, p.squash.commit.clone()]);
    assert_eq!(p.publish(&git), PublishAnswer::Published { origin: integration.clone() });
    assert_eq!(p.remote_tip(), Some(integration.clone())); assert_eq!(p.f.git.read_ref(MAIN).unwrap(), Some(integration));
    assert_eq!(p.trace_bytes(), b"push\n");
    let old = p.pushes().into_iter().filter(|m| m.record.op_id == id).collect::<Vec<_>>();
    assert_eq!(old.len(), 2, "relandoff must not close the old attempt a second time");
    assert_eq!(old[1].record.phase, OpPhase::Failed); p.assert_inert_repeat(&git);
}

#[test]
fn rit_remote_intent_absent_remote_is_failed_truthfully_and_retry_can_finish() {
    let p = Prepared::new(); let git = interrupt(&p, "before"); let id = assert_open(&p, &p.base);
    raw(&p.f, &["push", "--quiet", "origin", &format!(":{MAIN}")]); assert_eq!(p.remote_tip(), None);
    let state = std::fs::read(p.state()).unwrap(); recover_leaves_open(&p);
    let answer = with_lock(&p, || Ok::<_, oplog::OplogError>(land::publish(&git, &refs(), ACTOR)));
    assert!(matches!(answer, Err(PublishError::RemoteMissing { remote, refname }) if remote == "origin" && refname == MAIN));
    assert_closed(&p, &id, OpPhase::Failed, failed(None));
    assert_eq!(std::fs::read(p.state()).unwrap(), state); assert_eq!(p.remote_tip(), None);
    assert_eq!(p.f.git.read_ref(MAIN).unwrap(), Some(p.base.clone())); assert_eq!(p.trace_bytes(), b"");
    let log = p.pushes();
    let again = with_lock(&p, || Ok::<_, oplog::OplogError>(land::publish(&git, &refs(), ACTOR)));
    assert!(matches!(again, Err(PublishError::RemoteMissing { .. }))); assert_eq!(p.pushes(), log);
    raw(&p.f, &["push", "--quiet", "origin", &format!("{}:{MAIN}", p.base)]);
    assert_eq!(p.publish(&git), PublishAnswer::Published { origin: p.new.clone() });
    let records = p.pushes(); assert_eq!(records.len(), 4); let next = &records[2].record.op_id; assert_ne!(next, &id);
    assert_record(&p, &records[2], next, OpPhase::Intent, None);
    assert_record(&p, &records[3], next, OpPhase::Done, Some(OpResult::Moved));
    assert_eq!(p.trace_bytes(), b"push\n"); p.assert_finished(&p.new); p.assert_inert_repeat(&git);
}
