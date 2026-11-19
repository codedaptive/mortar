//! replay_record_ref: the ref recording which commit a replay made of a zone commit.
//! replay_record_ref_tests covers it.

/// `refs/mortar/replayed/<zone>/<source_hex>`: a Mortar hidden ref family, recording which commit
/// `refs/heads/zone/<zone>` holds for the commit `source_hex` after a replay. The op that moves the zone by a clean
/// replay writes it for every commit it replayed, and carries forward a record naming one of them; the publishing
/// zone's after_publish deletes every record at or behind the published squash source in the op that moves the
/// zone; when the zone is Current, a records-only op deletes the source's own record and, only when the zone holds
/// the source, the records at or behind it. Callers pass `oid.as_str()`; read_zone_records passes an empty
/// `source_hex` to list the zone's family.
pub(crate) fn replay_record_ref(zone: &str, source_hex: &str) -> String {
    format!("refs/mortar/replayed/{zone}/{source_hex}")
}
