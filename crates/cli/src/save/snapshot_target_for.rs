//! snapshot_target_for: the snapshot target a save in a folder writes to, read from the folder's record.

use folders::{FolderKind, FolderRecord};
use snapshot::SnapshotTarget;

/// The record's zone and request with `actor`, and the attempt from the record's kind: Some(k) for an attempt
/// folder k, so the save moves refs/mortar/trysnap/<zone>/<request>/<k> (S-69), and None for a request folder,
/// whose save moves the request's own snapshot ref. None when the record names no request (a zone folder).
pub fn snapshot_target_for(record: &FolderRecord, actor: &str) -> Option<SnapshotTarget> {
    let request = record.request.as_ref()?;
    let attempt = match record.kind {
        FolderKind::Attempt(k) => Some(k),
        FolderKind::Request | FolderKind::Zone => None,
    };
    Some(SnapshotTarget { zone: record.zone.clone(), request: request.clone(), attempt, actor: actor.to_string() })
}
