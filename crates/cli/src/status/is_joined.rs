//! is_joined: whether a request is joined to its zone.

use gitio::Oid;

use super::is_saved;

/// True when the zone holds the request's tip (`request_in_zone`), the request has commits of its own (a request
/// still at its `base` is in the zone trivially, without having joined anything), and the folder is saved
/// (is_saved: its tree, `folder_tree`, equals the tip's tree, `tip_tree`): a snapshot or edit after the join leaves the folder holding
/// work the zone does not have.
pub fn is_joined(base: &Oid, request_tip: &Oid, request_in_zone: bool, folder_tree: &Oid, tip_tree: &Oid) -> bool {
    request_in_zone && request_tip != base && is_saved(folder_tree, tip_tree)
}
