//! replay_records: the replay record moves a clean replay of a zone makes (rule W, S-121). replay_records_tests
//! covers it.

use crate::after_publish::{replay_record_ref, ZoneRecord};
use join::MappedCommit;
use oplog::RefMove;

/// Writes, in `mapped` order: for every mapped commit, its record `replay_record_ref(zone, source)` set to the
/// replayed commit, `old` the existing record's value from `records` (None when there is none). Then carries, in
/// `records` order: an existing record whose value is the source of a mapped commit set to that commit's replay, so
/// a record keeps naming the zone commit standing for its source. A ref a write moves is never carried (the write
/// wins: one move per ref), and a move whose old value already equals its new value is left out. Records whose
/// value is not a mapped source stay as they are.
pub(crate) fn replay_records(zone: &str, mapped: &[MappedCommit], records: &[ZoneRecord]) -> Vec<RefMove> {
    let written: Vec<String> = mapped.iter().map(|m| replay_record_ref(zone, m.source.as_str())).collect();
    let writes = mapped.iter().zip(&written).map(|(m, refname)| RefMove {
        refname: refname.clone(),
        old: records.iter().find(|r| &r.refname == refname).map(|r| r.commit.clone()),
        new: m.replayed.clone(),
    });
    let carries = records.iter().filter(|r| !written.contains(&r.refname)).filter_map(|r| {
        let m = mapped.iter().find(|m| m.source == r.commit)?;
        Some(RefMove { refname: r.refname.clone(), old: Some(r.commit.clone()), new: m.replayed.clone() })
    });
    writes.chain(carries).filter(|m| m.old.as_ref() != Some(&m.new)).collect()
}
