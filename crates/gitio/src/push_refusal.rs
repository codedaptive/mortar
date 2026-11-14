//! PushRefusal: why git refused to move one ref of a push. A declaration only; read_push_tests exercises it.

/// The reason git gave in the ref's push record, read from its summary and never guessed from other text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PushRefusal {
    /// `[rejected] (stale info)`: the remote's ref did not hold the lease's value.
    StaleLease,
    /// `[rejected] (non-fast-forward)`: an unleased move that is not a fast-forward.
    NonFastForward,
    /// `[remote rejected] (pre-receive hook declined)` or `(hook declined)`: a hook on the remote refused it.
    Hook,
    /// Any other summary, verbatim, such as `[rejected] (fetch first)` or `[remote rejected] (unpacker error)`.
    Other(String),
}
