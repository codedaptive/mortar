//! published_records: the replay record deletes a publish of a zone makes (rule D, S-121).
//! published_records_tests covers it.

use crate::after_publish::{replay_record_ref, ZoneRecord};
use gitio::{GitError, Oid};
use oplog::RefMove;

/// One delete, in `records` order, for every record of the published `source`: its ref is
/// `replay_record_ref(zone, source)` itself (whatever its value), or it is `behind` (at or behind the commit the
/// zone holds for the source, S-129; none is behind when it holds none). A delete's `old` is the record's value and
/// its `new` the all-zero id of that value's length (the oplog's delete, S-9). Every other record stays. A zero id that
/// does not parse is the Err.
pub(crate) fn published_records(zone: &str, source: &Oid, records: &[ZoneRecord]) -> Result<Vec<RefMove>, GitError> {
    let own = replay_record_ref(zone, source.as_str());
    records
        .iter()
        .filter(|r| r.behind || r.refname == own)
        .map(|r| {
            Ok(RefMove {
                refname: r.refname.clone(),
                old: Some(r.commit.clone()),
                new: Oid::parse(&"0".repeat(r.commit.as_str().len()))?,
            })
        })
        .collect()
}
