//! replay_record_ref: the ref recording which commit a replay made of a zone's squash source.
//! replay_record_ref_tests covers it.

/// `refs/mortar/replayed/<zone>/<source_hex>`: a Mortar hidden ref family (S-118), recording which
/// commit `refs/heads/zone/<zone>` holds for the squash source `source_hex` after a replay. The op that moves the
/// zone by a replay writes it, when the zone's squash names a source the replay carried; the publish that consumes
/// it deletes it, in the op that moves its zone past the source (S-118). Callers pass `oid.as_str()`.
pub(crate) fn replay_record_ref(zone: &str, source_hex: &str) -> String {
    format!("refs/mortar/replayed/{zone}/{source_hex}")
}
