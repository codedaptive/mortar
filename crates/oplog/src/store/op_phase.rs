//! Where an op stands in the log: announced, finished, or refused.
//! A bare type; encode_op_record_tests and decode_op_record_tests exercise each phase's JSON name.

/// JSON `"intent"` | `"done"` | `"failed"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpPhase {
    /// Written before the ref transaction, with every planned move.
    Intent,
    /// The refs moved.
    Done,
    /// The refs did not move.
    Failed,
}
