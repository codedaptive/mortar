//! Restored: what restore_backup did with one backup ref. A declaration only; restored_after_fetch_tests and the
//! backup integration tests (backup_zit) exercise it.

use gitio::Oid;

/// One backup ref against the local store. `refname` is always the local ref name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Restored {
    /// The local ref was absent and is now created at the backup's commit.
    Created { refname: String, oid: Oid },
    /// The local ref already holds the backup's commit.
    Same { refname: String },
    /// The local ref holds another commit; it is listed, not moved.
    Differs { refname: String, local: Oid, backup: Oid },
    /// Listed by ls-remote but gone from the remote by the fetch; nothing was created for it.
    Vanished { refname: String },
}
