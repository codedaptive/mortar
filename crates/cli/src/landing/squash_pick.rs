//! SquashPick: which squash ref carries a change ID, as pick_squash_ref answers it.
//! No sibling test: a bare type with no behaviour of its own; pick_squash_ref_tests cover how it is chosen.

use gitio::Oid;

/// One ref carries the change (its full refname and commit), none does, or several do (their refnames in git's order).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SquashPick {
    One { refname: String, commit: Oid },
    Missing,
    Several { refnames: Vec<String> },
}
