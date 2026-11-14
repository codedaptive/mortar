//! PushRef: one remote ref a push sets or deletes. A declaration only; push_request_tests and read_push_tests
//! exercise it.

use crate::{Oid, PushLease};

/// One ref to set on the remote, and the check made on its current value first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PushRef {
    /// The full ref name on the remote, such as `refs/heads/main`.
    pub refname: String,
    /// The commit to set the ref to; None deletes the ref.
    pub new: Option<Oid>,
    /// What the ref must hold on the remote for the push to change it.
    pub lease: PushLease,
}
