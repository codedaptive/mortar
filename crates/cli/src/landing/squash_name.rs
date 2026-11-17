//! SquashName: what a landing verb was told to read as a squash, a commit or a change ID.
//! No sibling test (S-7): a bare type with no behaviour of its own.

use gitio::Oid;
use oplog::ChangeId;

/// A squash named by its commit, or by the change ID one squash ref under refs/mortar/squash/ carries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SquashName {
    Commit(Oid),
    Change(ChangeId),
}
