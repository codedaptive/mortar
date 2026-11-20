//! settle_push_intent: on a repeat, publish closes a push intent an earlier run left open, from what the remote holds
//! now: recover leaves such an intent open, and only publish reads the remote.
//! No unit test: it reads and appends the op log. Its decision is push_intent_close (push_intent_close_tests).
//! Covered by the publish_zit tests publishing::an_open_push_intent_after_the_push_is_closed_done_on_the_repeat,
//! publishing::an_open_push_intent_before_the_push_is_closed_failed_and_pushed_again,
//! publishing::an_open_push_intent_with_the_remote_past_new_is_closed_done and
//! kills.rs killed_between_the_steps_recovers_and_the_repeat_publishes_once (nothing left open, nothing written).

use crate::landing::{publish_push_op, push_intent_close, LandingState, PublishError};
use crate::squash::LandRefs;
use gitio::Oid;
use oplog::{close_intent, open_intents, read_marked_log, OpRecord};
use std::path::Path;

/// The push key is that of the op publish_push_op builds for the state's move (expected = origin_at_start, new =
/// integration); the key holds no op id, actor or time, so placeholders stand in for them. The last open intent in
/// the log with that key and a remote mark is closed once through close_intent with push_intent_close(`remote`,
/// `landed`, the origin ref): Done when the push landed (the remote at or past new, as push_landed answers), else
/// Failed naming what the remote holds. No such intent: nothing is written. Reading or appending the log failing is
/// Oplog.
pub(crate) fn settle_push_intent(
    common: &Path,
    refs: &LandRefs,
    state: &LandingState,
    remote: Option<&Oid>,
    landed: bool,
) -> Result<(), PublishError> {
    let key = publish_push_op(refs, &state.origin_at_start, &state.integration, &state.change_id, String::new(), "", 0)
        .key;
    let marked = read_marked_log(common).map_err(PublishError::Oplog)?.records;
    let records: Vec<OpRecord> = marked.iter().map(|m| m.record.clone()).collect();
    let remote_marked = |op_id: &str| marked.iter().any(|m| m.record.op_id == op_id && m.remote.is_some());
    let Some(open) = open_intents(&records).into_iter().rev().find(|r| r.key == key && remote_marked(&r.op_id)) else {
        return Ok(());
    };
    let refname = format!("refs/heads/{}", refs.origin_branch);
    close_intent(common, &open.op_id, push_intent_close(remote, landed, &refname))
        .map_err(PublishError::Oplog)?;
    Ok(())
}
