//! read_git_replay: the new tip git's replay gave, if it gave one.

use crate::JoinError;
use gitio::{Oid, ReplayOutcome};

/// Replayed: the new value of the update for `tip_ref`; updates none of which is for `tip_ref` are refused as
/// TipNotReplayed. No updates at all: None, so the commit-by-commit replay runs, because git can replay commits and
/// print nothing, so empty output does not prove the range was empty (that replay's rev_list decides it).
/// Conflicted: None, because git names no commit, tree or files for it, so the caller replays commit by commit to
/// find them.
pub(crate) fn read_git_replay(outcome: ReplayOutcome, tip_ref: &str) -> Result<Option<Oid>, JoinError> {
    let updates = match outcome {
        ReplayOutcome::Replayed(updates) if !updates.is_empty() => updates,
        ReplayOutcome::Replayed(_) | ReplayOutcome::Conflicted => return Ok(None),
    };
    updates
        .into_iter()
        .find(|update| update.refname == tip_ref)
        .map(|update| Some(update.new))
        .ok_or_else(|| JoinError::TipNotReplayed(tip_ref.to_string()))
}
