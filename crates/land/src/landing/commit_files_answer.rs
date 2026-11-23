//! CommitFilesAnswer: what commit_files did. A bare type; tests/commit_files_zit (committing.rs) reads both variants.

use gitio::Oid;

/// commit_files' answer when it was not refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommitFilesAnswer {
    /// One commit holding the named paths on top of integration's tip; integration and the landing state now name it,
    /// so publish pushes it with the landing and back_out drops it with the landing.
    Committed {
        /// The commit integration was moved to.
        commit: Oid,
    },
    /// The named paths already stood in integration's tip as in the folder: no commit, no op, nothing moved.
    Same,
}
