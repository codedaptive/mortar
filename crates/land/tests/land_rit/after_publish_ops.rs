//! after_publish -> actual Gitio movement and Oplog Intent/Done; zone_move_op is reached only publicly (S-109).
use crate::{owed_ops::{locked, raw, refs}, push_fixture::{Prepared, ACTOR, ZONE}};
use land::{PublishAnswer, ZoneMove, ZoneRequests};
use oplog::{OpPhase, OpRecord, OpResult, RefMove};
use std::time::{SystemTime, UNIX_EPOCH};

fn now() -> u64 { SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() }
fn move_zone(p: &Prepared) {
    let expected = vec![ZoneMove::Moved { zone: "core".into(), old: p.f.source.clone(), new: p.new.clone(), rebase: vec![] }];
    let zone = ZoneRequests { zone: "core".into(), requests: vec![] };
    assert_eq!(locked(&p.f, || land::after_publish(&p.f.git, &refs(), &zone, &[], ACTOR)), expected);
    assert_eq!(p.f.git.read_ref(ZONE).unwrap(), Some(p.new.clone()));
}
fn records(p: &Prepared) -> Vec<OpRecord> {
    oplog::read_op_log(&p.f.common).unwrap().records.into_iter().filter(|r| r.kind == "after-publish").collect()
}
fn assert_pair(p: &Prepared, pair: &[OpRecord], before: u64, after: u64) {
    assert_eq!(pair.len(), 2); assert!(!pair[0].op_id.is_empty());
    let key = format!("after-publish:{ZONE}:{}:{}:{}", p.f.source, p.new, pair[0].op_id);
    let reason = format!("after-publish moves {ZONE} from {} to {}", p.f.source, p.new);
    for (r, (phase, result)) in pair.iter().zip([(OpPhase::Intent, None), (OpPhase::Done, Some(OpResult::Moved))]) {
        // Independent surrounding clock reads bound metadata; no speed is measured.
        assert!((before..=after).contains(&r.at));
        assert_eq!(*r, OpRecord { op_id: pair[0].op_id.clone(), key: key.clone(), kind: "after-publish".into(),
            actor: ACTOR.into(), reason: reason.clone(), at: r.at, phase,
            moves: vec![RefMove { refname: ZONE.into(), old: Some(p.f.source.clone()), new: p.new.clone() }], result });
    }
}
#[test]
fn rit_after_publish_key_reason_and_metadata_name_the_actual_zone_move() {
    let p = Prepared::new(); assert_eq!(p.publish(&p.f.git), PublishAnswer::Published { origin: p.new.clone() });
    // Origin/new is known independently from the earlier public squash/land/publish, not from ZoneMove's answer.
    let before = now(); move_zone(&p); let after = now(); assert_pair(&p, &records(&p), before, after);
    let marked = oplog::read_marked_log(&p.f.common).unwrap().records;
    assert!(marked.iter().filter(|m| m.record.kind == "after-publish").all(|m| m.remote.is_none() && m.values.is_empty()));
}
#[test]
fn rit_after_publish_identical_move_inputs_get_fresh_attempt_ids_and_keys() {
    let p = Prepared::new(); assert_eq!(p.publish(&p.f.git), PublishAnswer::Published { origin: p.new.clone() });
    let before = now(); move_zone(&p);
    // Only this disposable fixture's zone is restored, so both PUBLIC attempts have the same old/new inputs.
    raw(&p.f, &["update-ref", ZONE, p.f.source.as_str()]); move_zone(&p); let after = now();
    let log = records(&p); assert_eq!(log.len(), 4);
    assert_pair(&p, &log[..2], before, after); assert_pair(&p, &log[2..], before, after);
    assert_ne!(log[0].op_id, log[2].op_id); assert_ne!(log[0].key, log[2].key);
    assert_eq!(log[0].moves, log[2].moves); assert_eq!(log[0].reason, log[2].reason);
}
