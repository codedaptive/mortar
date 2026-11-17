//! find_zone_old: the commit a zone folder stands on, found by its index's tree among the values its zone branch has
//! held (S-140).

use gitio::Oid;

use super::ZoneOld;

/// AtTip when `index_tree` is `tip_tree`, reading no tree. Else the first of `values`, in the order given, whose tree
/// (`tree_of`) is `index_tree`: From(that value), reading no value after it. A value whose tree `tree_of` cannot give
/// (None) is passed over. NoMatch when no value has that tree.
pub fn find_zone_old(
    index_tree: &Oid,
    tip_tree: &Oid,
    values: &[Oid],
    mut tree_of: impl FnMut(&Oid) -> Option<Oid>,
) -> ZoneOld {
    if index_tree == tip_tree {
        return ZoneOld::AtTip;
    }
    values
        .iter()
        .find(|value| tree_of(value).as_ref() == Some(index_tree))
        .map_or(ZoneOld::NoMatch, |value| ZoneOld::From(value.clone()))
}
