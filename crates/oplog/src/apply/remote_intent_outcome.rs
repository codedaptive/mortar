//! What opening a remote intent did.
//! A bare type, no unit test; crates/oplog/tests/oplog/remote_verdict.rs exercises the AlreadyOpen and Repeated variants.

use crate::store::MarkedRecord;

/// The answer of open_remote_intent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RemoteIntentOutcome {
    /// The intent record just appended, marked with the remote, with no values.
    Opened(MarkedRecord),
    /// An open intent with this key, marked with a remote, is already in the log; nothing was appended. The
    /// caller settles it: it reads the remote, then closes the intent with close_intent.
    AlreadyOpen(MarkedRecord),
    /// A done record with this key is in the log: the op already ran and was closed done; nothing was appended.
    Repeated(MarkedRecord),
}
