//! Settles every open intent in the op log: reads its refs, decides, appends the closing record.
//! No unit test: it reads the log, runs git and appends to the log (S-5); the decisions are covered by
//! open_intents_tests, decide_recovery_tests, recovery_close_tests and settled_or_first_error_tests.
//! Nagatha's integration group, in a throwaway repository, should cover: an empty or missing log returns no records; an intent whose refs
//! all moved gets a done/moved record, one whose refs never moved a failed/not-moved record, each read
//! back by read_op_log; an intent already closed gets nothing; a mixed intent is OplogError::Mixed naming
//! its op_id, key and refs, appends nothing for it, and the other open intents are still settled; a
//! second recover after the first appends nothing; a kill between apply's transaction and its done
//! append is settled as done.

use std::path::Path;

use gitio::Git;

use super::{decide_recovery, open_intents, recovery_close, settled_or_first_error};
use crate::store::{append_op_record, read_op_log, read_ref_value, unix_now, OpRecord, OplogError};

/// The closing records appended, in log order; after settling every other intent, the first error met.
pub fn recover(git: &Git, common: &Path) -> Result<Vec<OpRecord>, OplogError> {
    let results = open_intents(&read_op_log(common)?.records)
        .into_iter()
        .map(|intent| settle(git, common, intent))
        .collect();
    settled_or_first_error(results)
}

/// Reads each move's ref, closes the intent by the decision and appends the record.
fn settle(git: &Git, common: &Path, intent: &OpRecord) -> Result<OpRecord, OplogError> {
    let refs = intent
        .moves
        .iter()
        .map(|m| Ok((m.clone(), read_ref_value(git, &m.refname)?)))
        .collect::<Result<Vec<_>, OplogError>>()?;
    let closing = recovery_close(intent, decide_recovery(&refs), unix_now())?;
    append_op_record(common, &closing)?;
    Ok(closing)
}
