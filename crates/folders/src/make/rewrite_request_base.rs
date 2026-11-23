//! Rewrites a request folder's record to a new base and partners, everything else in it as it was.
//! No unit test: it reads and writes a real record file; encode_record_and_partners_tests and
//! decode_record_and_partners_tests cover the text. tests/folders/partner_base.rs covers, in a throwaway repository: a rewrite to a merge base with
//! a partner and back to the zone tip with none changes base and partners only, every other record byte the same,
//! and leaves the request branch where it was; a zone folder refuses with ZoneFolder and its record is unchanged.

use std::path::Path;

use gitio::Git;

use super::read_record_and_partners::read_record_and_partners;
use super::{write_folder_record, FolderError};
use crate::plan::zone_branch;
use crate::record::{FolderKind, FolderRecord, RecordedBase};

/// Writes the record of the request folder at `folder` with base.base as its base and base.partners as its
/// partners, every other field as read, through write_folder_record (whole or not at all), and returns the new
/// record. Each partner's zone name is checked first (zone_branch), refused as Plan. Refuses a worktree with no
/// record as NotAFolder, a zone folder as ZoneFolder and an attempt folder as AttemptFolder, with nothing written.
/// Moves no ref and touches no file in the folder.
///
/// Takes no lock: the caller holds the join lock of the folder's zone (join-<zone>) while it rewrites.
pub fn rewrite_request_base(git: &Git, folder: &Path, base: &RecordedBase) -> Result<FolderRecord, FolderError> {
    for partner in &base.partners {
        zone_branch(&partner.zone).map_err(FolderError::Plan)?;
    }
    let git_dir = git.git_dir(folder).map_err(FolderError::Git)?;
    let Some((existing, _)) = read_record_and_partners(&git_dir)? else {
        return Err(FolderError::NotAFolder {
            path: folder.to_path_buf(),
            reason: "a worktree with no Mortar folder record".to_string(),
        });
    };
    match existing.kind {
        FolderKind::Zone => {
            return Err(FolderError::ZoneFolder {
                path: folder.to_path_buf(),
                zone: existing.zone,
            })
        }
        FolderKind::Attempt(k) => return Err(FolderError::AttemptFolder { path: folder.to_path_buf(), k }),
        FolderKind::Request => {}
    }
    let record = FolderRecord {
        base: base.base.clone(),
        ..existing
    };
    write_folder_record(&git_dir, &record, &base.partners)?;
    Ok(record)
}
