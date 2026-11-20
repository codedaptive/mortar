//! How the caller closes an open intent with close_intent.
//! A bare type; crates/oplog/tests/oplog/remote_close_record.rs exercises it through close_intent.

use gitio::Oid;

/// The outcome the caller saw for the op.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntentClose {
    /// The op's moves were made: the closing record is done, its result Moved.
    Done,
    /// Nothing moved: each ref with the value it holds now (None: the ref is absent), as recover's failed
    /// records carry them. The closing record is failed, its result NotMoved { refs }.
    Failed { refs: Vec<(String, Option<Oid>)> },
}
