//! read_push_intent: where this publish's push intent stands in the op log.
//! No unit test: it reads the op log (S-5). Covered by tests/publish_zit: repeats.rs
//! open_push_intent_refuses_push_off_then_push_on_finishes (Open, then Done) and
//! push_off_killed_after_origin_move_finishes_push_on_without_push (Absent); publishing.rs
//! publish_moves_the_remote_then_the_local_origin_and_cleans_up (Done) and the open-intent cases through
//! settle_push_intent.

use crate::landing::{publish_push_op, LandingState, PublishError, PushIntentState};
use crate::squash::LandRefs;
use oplog::{open_intents, read_marked_log, OpPhase, OpRecord};
use std::path::Path;

/// The push key is that of the op publish_push_op builds for the state's move (expected = origin_at_start, new =
/// integration); the key holds no op id, actor or time, so placeholders stand in for them. Among the log's records
/// with that key and a remote mark: the last open intent is Open { op_id }; else any done record is Done; else
/// Absent, a history of failed pushes only included. Reading the log failing is Oplog.
pub(crate) fn read_push_intent(
    common: &Path,
    refs: &LandRefs,
    state: &LandingState,
) -> Result<PushIntentState, PublishError> {
    let key = publish_push_op(refs, &state.origin_at_start, &state.integration, &state.change_id, String::new(), "", 0)
        .key;
    let marked = read_marked_log(common).map_err(PublishError::Oplog)?.records;
    let records: Vec<OpRecord> =
        marked.into_iter().filter(|m| m.remote.is_some() && m.record.key == key).map(|m| m.record).collect();
    if let Some(open) = open_intents(&records).last() {
        return Ok(PushIntentState::Open { op_id: open.op_id.clone() });
    }
    Ok(if records.iter().any(|r| r.phase == OpPhase::Done) { PushIntentState::Done } else { PushIntentState::Absent })
}
