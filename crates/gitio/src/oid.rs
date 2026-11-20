//! Oid: a validated git object id.

/// 40 (sha1) or 64 (sha256) lowercase hex characters; made outside this crate only by Oid::parse.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Oid(pub(crate) String);
