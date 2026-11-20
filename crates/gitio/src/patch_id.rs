//! PatchId: the stable patch ID git computes for a change.

/// 40 (sha1) or 64 (sha256) lowercase hex characters; made outside this crate only by PatchId::parse. A type apart
/// from Oid because it names a change, not an object: the same change on two parents has one patch ID.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PatchId(pub(crate) String);
