//! requests_to_rebase: the open requests a zone move leaves behind. requests_to_rebase_tests covers it.

use gitio::Oid;
use join::OpenRequest;

/// The requests whose base is not the zone's new `tip`, in the order given: they rebase at their next join.
pub(crate) fn requests_to_rebase(requests: &[OpenRequest], tip: &Oid) -> Vec<OpenRequest> {
    requests.iter().filter(|request| request.base != *tip).cloned().collect()
}
