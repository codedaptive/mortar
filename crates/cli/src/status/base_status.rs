//! What a request folder stands on, as `mr status` reports it: the record's base, its partners and which of them
//! moved since.
//! A bare type; with_base_status_tests exercises it.

use folders::RequestPartner;
use gitio::Oid;

/// `base` is the folder record's base; `partners` the partner zones merged into it, in record order, empty for a
/// plain request; `moved` the names of the partners whose zone tip is no longer the recorded tip, in record order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaseStatus {
    pub base: Oid,
    pub partners: Vec<RequestPartner>,
    pub moved: Vec<String>,
}
