//! zone_values: the values a zone branch has held, newest first, for finding the commit its folder stands on (S-140).

use gitio::Oid;
use oplog::{OpPhase, OpRecord, RefMove};

/// `tip` first; then, walking `records` (the op log, oldest first) from the newest, each move of `zone_ref` in a
/// Done record: its new value, then its old one when it has one. Intent and Failed records are skipped, since a
/// ref they name may never have held their values. Each value is listed once, at its first place.
pub fn zone_values(records: &[OpRecord], zone_ref: &str, tip: &Oid) -> Vec<Oid> {
    let mut values = vec![tip.clone()];
    let moves = records.iter().rev().filter(|record| is_done(record.phase)).flat_map(|record| &record.moves);
    for one in moves.filter(|one: &&RefMove| one.refname == zone_ref) {
        for value in std::iter::once(&one.new).chain(one.old.as_ref()) {
            if !values.contains(value) {
                values.push(value.clone());
            }
        }
    }
    values
}

/// Whether a record in `phase` moved its refs.
fn is_done(phase: OpPhase) -> bool {
    match phase {
        OpPhase::Done => true,
        OpPhase::Intent | OpPhase::Failed => false,
    }
}
