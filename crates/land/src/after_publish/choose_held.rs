//! choose_held: the zone commit standing for a squash source, from what was already read. choose_held_tests covers
//! every branch.

use gitio::Oid;

/// `recorded`: the commit the source's replay record names and whether the zone tip holds it, None without a record.
/// `source_held`: None when the source object is missing, else whether the tip holds it. The recorded commit when
/// the tip holds it; else `source` when the tip holds it; else None. The record comes first: after a replay it names
/// what the zone holds for the source, which the source itself no longer is.
pub(crate) fn choose_held(source: &Oid, recorded: Option<(Oid, bool)>, source_held: Option<bool>) -> Option<Oid> {
    match (recorded, source_held) {
        (Some((recorded, true)), _) => Some(recorded),
        (_, Some(true)) => Some(source.clone()),
        _ => None,
    }
}
