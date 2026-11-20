//! The git reads mr overlap's answer replaces.

use gitio::BaselineQuery;
use ::join::OpenRequest;

/// One `git log -p <base>..<tip>` over the whole tree per open request, in the order given: each request's own
/// commits since its fork point from the zone. No requests gives no queries.
pub fn overlap_queries(requests: &[OpenRequest]) -> Vec<BaselineQuery> {
    requests
        .iter()
        .map(|request| BaselineQuery::LogPatches { range: format!("{}..{}", request.base, request.tip), paths: vec![] })
        .collect()
}
