//! An orphan attempt's snapshot, as remake_plan reads it.
//! A bare type; remake_plan_tests exercises it.

use gitio::Oid;

/// The commit an attempt's trysnap ref names: `tree`, its tree; `first_parent`, its first parent, None when the
/// commit has no parent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snap {
    pub tree: Oid,
    pub first_parent: Option<Oid>,
}
