//! The all-zero id that deletes a ref (S-9), as long as a given id: 40 for SHA-1, 64 for SHA-256.

use gitio::Oid;

use crate::store::OplogError;

/// All zeros, the length of `oid`. The Err is Git carrying Oid::parse's refusal of the zeros text, which
/// zeros of an Oid's own length (40 or 64) never get.
pub(crate) fn zero_oid_like(oid: &Oid) -> Result<Oid, OplogError> {
    Oid::parse(&"0".repeat(oid.as_str().len())).map_err(OplogError::Git)
}
