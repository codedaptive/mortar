//! is_joined: whether a request is joined to its zone.

use gitio::Oid;

/// True when the zone holds the request's tip (`request_in_zone`) and the request has commits of its own: a
/// request still at its `base` is in the zone trivially, without having joined anything.
pub fn is_joined(base: &Oid, request_tip: &Oid, request_in_zone: bool) -> bool {
    request_in_zone && request_tip != base
}
