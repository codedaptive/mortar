//! Opens a remote intent: the repeat check, the ref name check, then the intent marked with the remote; no ref
//! transaction runs.
//! No unit test: it reads and appends the log file and runs git (S-5). crates/oplog/tests/oplog/remote_verdict.rs
//! covers its decision on a seeded log. The integration tests in crates/oplog/tests/oplog/remote_intents.rs cover,
//! in a throwaway repository: a fresh op opens an intent with no ref moving, which reads back marked with the remote; a
//! second call with a key whose marked intent is still open answers AlreadyOpen and appends nothing; a key
//! already done answers Repeated; a bad ref name is OplogError::BadMove with nothing written.

use std::path::Path;

use gitio::Git;

use super::{check_ref_names, remote_intent_verdict, RemoteIntentOutcome};
use crate::store::{append_marked_record, intent_record, read_marked_log, MarkedRecord, Op, OplogError};

/// Writes the intent record for `op` to the log at `<common>/mortar/oplog.jsonl`, marked with `remote` (the
/// remote's name) and with no values. It runs no ref transaction: the moves happen on the remote.
///
/// The caller holds the lock and has run recover first (S-16). recover leaves this intent open; the caller
/// closes it with close_intent once it knows the remote's answer (S-103).
///
/// A key already done answers Repeated, and a key whose marked intent is still open answers AlreadyOpen;
/// both write nothing (remote_intent_verdict). Otherwise a move whose ref name is unusable returns BadMove,
/// and a git failure while checking returns OplogError::Git, before anything is written.
pub fn open_remote_intent(
    git: &Git,
    common: &Path,
    op: &Op,
    remote: &str,
) -> Result<RemoteIntentOutcome, OplogError> {
    if let Some(outcome) = remote_intent_verdict(&read_marked_log(common)?.records, &op.key) {
        return Ok(outcome);
    }
    check_ref_names(git, &op.moves)?;
    let intent = intent_record(op);
    append_marked_record(common, &intent, &[], Some(remote))?;
    Ok(RemoteIntentOutcome::Opened(MarkedRecord {
        record: intent,
        values: vec![],
        remote: Some(remote.to_string()),
    }))
}
