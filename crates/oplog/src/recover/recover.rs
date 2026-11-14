//! Settles every open intent in the op log not marked with a remote: reads its refs, decides, appends the
//! closing record, which carries the intent's values. An intent marked with a remote (open_remote_intent) is
//! neither read nor closed: not failed, not unknown, nothing appended; its caller closes it with
//! close_intent.
//! No unit test: it reads the log, runs git and appends to the log (S-5); the decisions are covered by
//! open_intents_in_tests, decide_recovery_tests, recovery_close_valued_tests and
//! settled_or_first_error_tests, and which intents it settles by crates/oplog/tests/oplog/recoverable.rs.
//! Nagatha's integration group, in a throwaway repository, should cover: an empty or missing log returns
//! no records; an intent whose changing moves are all at new (unchanged moves never decide) gets a
//! done/moved record, one with no changing ref at new a failed/not-moved record naming each ref and its
//! actual value, each read back by read_op_log; an intent already closed gets nothing; a true mix (a
//! changing ref at new beside a changing ref not at new) is OplogError::Mixed naming its op_id, key and
//! refs, appends nothing for it, and the other open intents are still settled; a second recover after the
//! first appends nothing; a kill between apply's transaction and its done append is settled as done; a
//! kill after a Stale answer, then recover, is failed recording the actual values for an op that changes a
//! ref, and done for one that changes none; an intent with a ref moved by a foreign actor to a third value
//! and no changing ref at new is failed; an intent written with values is closed, done or failed, by a
//! record that read_valued_log returns with the same values in the same order.
//! crates/oplog/tests/oplog/remote_intents.rs covers a marked intent left open beside a settled unmarked one.

use std::path::Path;

use gitio::Git;

use super::{decide_recovery, recoverable_intents, recovery_close_valued, settled_or_first_error};
use crate::store::{
    append_valued_record, read_marked_log, read_ref_value, unix_now, OpRecord, OplogError, ValuedRecord,
};

/// The closing records appended, in log order; after settling every other intent, the first error met.
/// Intents marked with a remote are skipped.
pub fn recover(git: &Git, common: &Path) -> Result<Vec<OpRecord>, OplogError> {
    let results = recoverable_intents(&read_marked_log(common)?.records)
        .iter()
        .map(|intent| settle(git, common, intent))
        .collect();
    settled_or_first_error(results)
}

/// Reads each move's ref, closes the intent by the decision with its values and appends the record.
fn settle(git: &Git, common: &Path, intent: &ValuedRecord) -> Result<OpRecord, OplogError> {
    let refs = intent
        .record
        .moves
        .iter()
        .map(|m| Ok((m.clone(), read_ref_value(git, &m.refname)?)))
        .collect::<Result<Vec<_>, OplogError>>()?;
    let closing = recovery_close_valued(intent, decide_recovery(&refs), unix_now())?;
    append_valued_record(common, &closing.record, &closing.values)?;
    Ok(closing.record)
}
