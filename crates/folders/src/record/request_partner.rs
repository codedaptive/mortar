//! One partner zone merged into a request folder's base.
//! A bare type; encode_record_and_partners_tests and decode_record_and_partners_tests exercise it.

use gitio::Oid;

/// The partner zone `zone` and the commit `tip` of it that was merged in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestPartner {
    pub zone: String,
    pub tip: Oid,
}
