//! Which commit a zone folder starts on, from the zone branch's tip and the caller's start commit.

use gitio::Oid;

use super::{FolderError, ZoneStart};

/// Existing(tip) when the zone branch has a tip, whatever `start` says; Create(start) when it has none and a
/// start is given; else ZoneBranchMissing naming `branch` (S-27).
pub(crate) fn zone_start_for(tip: Option<&Oid>, start: Option<&Oid>, branch: &str) -> Result<ZoneStart, FolderError> {
    match (tip, start) {
        (Some(tip), _) => Ok(ZoneStart::Existing(tip.clone())),
        (None, Some(start)) => Ok(ZoneStart::Create(start.clone())),
        (None, None) => Err(FolderError::ZoneBranchMissing {
            branch: branch.to_string(),
        }),
    }
}
