//! Decides, from the marked log alone, whether opening a remote intent for a key would repeat earlier work.
//! No unit test: its decisions run through find_done_in and open_intents_in.
//! crates/oplog/tests/oplog/remote_verdict.rs proves them through open_remote_intent on a seeded log.

use super::{find_done_in, RemoteIntentOutcome};
use crate::recover::open_intents_in;
use crate::store::MarkedRecord;

/// Repeated with the first done record with `key`; else AlreadyOpen with the last open intent with `key`
/// that is marked with a remote; else None, and the caller opens a new intent.
///
/// A failed record with the key does not count: the op may run again, as in apply. An open unmarked intent
/// with the key does not count either: recover settles those, and callers run recover first.
pub(crate) fn remote_intent_verdict(records: &[MarkedRecord], key: &str) -> Option<RemoteIntentOutcome> {
    if let Some(done) = find_done_in(records, key, |m| &m.record) {
        return Some(RemoteIntentOutcome::Repeated(done.clone()));
    }
    open_intents_in(records, |m| &m.record)
        .into_iter()
        .rev()
        .find(|m| m.record.key == key && m.remote.is_some())
        .map(|m| RemoteIntentOutcome::AlreadyOpen(m.clone()))
}
