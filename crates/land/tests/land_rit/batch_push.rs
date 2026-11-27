//! Batch public publish -> Oplog marked remote intent -> real SIGKILL/recovery/repeat across the push boundary.
use crate::{batch_fixture::IDS, push_fixture::{Prepared, ACTOR, MAIN}, push_kill::{interrupt, recover_leaves_open}};
use land::PublishAnswer;
use oplog::{MarkedRecord, OpPhase, OpRecord, OpResult, RefMove};
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn rit_batch_push_intent_spans_origin_to_newest_integration_across_actual_kills() {
    for mode in ["before", "after"] {
        let p = Prepared::batch(); assert_ne!(p.base, p.squash.commit); assert_ne!(p.new, p.squash.commit);
        let git = interrupt(&p, mode); let open = p.pushes(); assert_eq!(open.len(), 1);
        assert_eq!(p.remote_tip(), Some(if mode == "before" { p.base.clone() } else { p.new.clone() }));
        assert_eq!(p.f.git.read_ref(MAIN).unwrap(), Some(p.base.clone())); assert!(p.state().is_file());
        recover_leaves_open(&p); assert_eq!(p.pushes(), open);
        assert_eq!(p.publish(&git), PublishAnswer::Published { origin: p.new.clone() });
        let records = p.pushes();
        let phases = if mode == "before" {
            vec![(OpPhase::Intent, None), (OpPhase::Failed, Some(OpResult::NotMoved { refs: vec![(MAIN.into(), Some(p.base.clone()))] })),
                (OpPhase::Intent, None), (OpPhase::Done, Some(OpResult::Moved))]
        } else { vec![(OpPhase::Intent, None), (OpPhase::Done, Some(OpResult::Moved))] };
        assert_eq!(records.len(), phases.len()); let first_id = &open[0].record.op_id; assert!(!first_id.is_empty());
        if mode == "before" { assert_ne!(&records[2].record.op_id, first_id); }
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        for (i, (actual, (phase, result))) in records.iter().zip(phases).enumerate() {
            let id = if i < 2 { first_id } else { &records[2].record.op_id };
            assert!(actual.record.at > 0 && actual.record.at <= now);
            assert_eq!(*actual, MarkedRecord { record: OpRecord {
                op_id: id.clone(), key: format!("publish-push:origin:{MAIN}:{}:{}", p.base, p.new),
                kind: "publish-push".into(), actor: ACTOR.into(),
                reason: format!("publish-push {} moves {MAIN} from {} to {}", IDS[1], p.base, p.new),
                at: actual.record.at, phase, result,
                moves: vec![RefMove { refname: MAIN.into(), old: Some(p.base.clone()), new: p.new.clone() }],
            }, values: vec![], remote: Some("origin".into()) });
        }
        assert_eq!(p.trace_bytes(), b"push\n"); p.assert_finished(&p.new);
        assert_eq!(p.f.git.read_ref("refs/mortar/squash/second").unwrap(), None); p.assert_inert_repeat(&git);
    }
}
