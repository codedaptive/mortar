//! Where a zone folder starts: on its existing branch, or on a branch to create.
//! A bare type; zone_start_for_tests exercises both values.

use gitio::Oid;

/// `Existing(tip)`: zone/<zone> exists at `tip` and is used as it is. `Create(start)`: zone/<zone> is absent
/// and is made at `start` through the op log.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ZoneStart {
    Existing(Oid),
    Create(Oid),
}
