//! ReplayOutcome: what replaying a range of commits onto a new base gave. A declaration only; read_replay_tests
//! exercises it.

use crate::RefUpdate;

/// The ref moves the replay asks for, or a conflict.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayOutcome {
    /// The replay was clean. Each update names the ref, the replayed commit it should move to and the value it held
    /// when git read it; nothing has been moved (pass them to `Git::update_refs`).
    Replayed(Vec<RefUpdate>),
    /// A commit did not apply cleanly; no ref is to be moved.
    Conflicted,
}
