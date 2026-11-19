//! ZoneRecord: one replay record of a zone, as read for after_publish. A declaration only;
//! replay_records_tests and published_records_tests exercise it.

use gitio::Oid;

/// `refname` is the record's ref, `refs/mortar/replayed/<zone>/<source hex>` (replay_record_ref); `commit` its
/// value, the zone commit standing for that source. `behind` is true when the record is at or behind the zone commit
/// standing for the published squash source (its value is an ancestor of that commit, or that commit itself); false
/// when that was not computed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ZoneRecord {
    pub refname: String,
    pub commit: Oid,
    pub behind: bool,
}
