//! PushLease: what a pushed ref must hold on the remote for the push to move it. A declaration only;
//! push_request_tests exercises it.

use crate::Oid;

/// The check git makes on the remote's current value of one ref before moving it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PushLease {
    /// No check: git's fast-forward rule alone decides, so a move that is not a fast-forward is refused.
    Unleased,
    /// The ref must not exist on the remote; an existing ref is refused as a stale lease.
    Absent,
    /// The ref must hold this value on the remote; then any move is allowed, fast-forward or not.
    At(Oid),
}
