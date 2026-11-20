//! PushOutcome: what a push did to one requested ref. A declaration only; read_push_tests exercises it.

use crate::PushRefusal;

/// The answer for one ref of a push.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PushOutcome {
    /// The remote's ref was created, moved or deleted.
    Pushed,
    /// The remote's ref already held the value; nothing was sent for it.
    UpToDate,
    /// Git refused to move the ref, for the reason given.
    Refused(PushRefusal),
}
