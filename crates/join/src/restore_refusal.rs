//! restore_refusal: the refusals restore_join_state would give that need no ref read, checked in its order: the
//! record first, then the state file. The record and state-file decisions are passed in, so restore_refusal_tests
//! proves its own rules with fakes: the record is checked before any read, the state is read once, the record's and
//! the read's errors pass through, StatePresent carries the path read, and Write and AlreadyRestored are Ok. The
//! record decision is proved by recorded_join_state_tests, the state-file decision by restore_state_step_tests; the
//! chain with the real deciders in a real repository is owed as RITs (check_restore_join_state.rs header).

use crate::join_state::JoinState;
use crate::restore_step::RestoreStep;
use crate::JoinRunError;
use oplog::ValuedRecord;
use std::path::PathBuf;

/// Ok unless a restore of `op` would be refused for the record or the state file. `record` decides the state `op`
/// recorded, and its error (NotAResolveOp, RecordedStateCorrupt) is returned before `read_state` is called;
/// `read_state` is then called once for the state file's path and contents, and its errors pass through;
/// `state_step` decides from the state read and the recorded state. StatePresent is refused at the path read; Write
/// and AlreadyRestored are both Ok. The request ref is not read, so it is never a refusal here.
pub(crate) fn restore_refusal<D, R, S>(
    op: &ValuedRecord,
    record: D,
    read_state: R,
    state_step: S,
) -> Result<(), JoinRunError>
where
    D: FnOnce(&ValuedRecord) -> Result<JoinState, JoinRunError>,
    R: FnOnce() -> Result<(PathBuf, Option<JoinState>), JoinRunError>,
    S: FnOnce(Option<&JoinState>, &JoinState) -> RestoreStep,
{
    let recorded = record(op)?;
    let (path, existing) = read_state()?;
    match state_step(existing.as_ref(), &recorded) {
        RestoreStep::Write | RestoreStep::AlreadyRestored => Ok(()),
        RestoreStep::StatePresent => Err(JoinRunError::StatePresent { state: path }),
        // The only caller, check_restore_join_state, passes restore_state_step, which leaves the request ref to
        // decide_restore and never answers Stale.
        RestoreStep::Stale { .. } => unreachable!("the state step passed in never answers Stale"),
    }
}
