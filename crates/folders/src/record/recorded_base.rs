//! What a request folder stands on, as its record says.
//! A bare type; the partner_base integration tests exercise it.

use gitio::Oid;

use super::RequestPartner;

/// `base` is the commit the request folder stands on; `partners` are the partner zones merged into it, in the
/// order they were merged, and empty for a plain request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedBase {
    pub base: Oid,
    pub partners: Vec<RequestPartner>,
}
