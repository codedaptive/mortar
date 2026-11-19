//! What sweep_refs does with one ref its caller named.
//! A bare type; extra_ref_verdict_tests exercises it.

use gitio::Oid;

/// Accept: the ref is deleted from this id. Skip: the ref is absent, so there is nothing to delete. Refuse: the
/// ref is never deleted by a sweep, for the reason given (a phrase that follows the ref's name).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtraRef {
    Accept(Oid),
    Skip,
    Refuse(String),
}
