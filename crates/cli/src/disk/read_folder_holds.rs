//! read_folder_holds: what one folder `mr disk` lists holds: its unsaved counts and whether its request has joined.
//! No unit test: it runs git; joined_from_reads_tests covers the joined rule and joined_unknown_reason_tests the
//! reason of an unknown answer, and the integration tests
//! crates/cli/tests/across_kits/snapshot_overlap_disk_sweep/disk.rs (disk_holds_dirty_request_counts_without_writing,
//! disk_holds_clean_saved_and_joined_request, disk_holds_zone_counts_without_a_joined_answer,
//! disk_answers_every_folder_when_one_joined_read_fails) cover what it reads.

use folders::{folder_unsaved, FolderDisk, FolderKind};
use gitio::Git;

use super::{joined_unknown_reason, FolderHolds, JoinedAnswer};
use crate::config::SnapshotSettings;
use crate::report::Report;
use crate::status::{joined_from_reads, read_branch_reads};
use crate::upkeep::folders_error_report;

const VERB: &str = "disk";

/// The folder's unsaved counts (folders::folder_unsaved; its Err is folders_error_report for disk, the one Err) and
/// its joined answer: ZoneFolder for a zone folder; for a request or attempt folder, Joined or NotJoined by
/// joined_from_reads over read_branch_reads, run in the folder with `git`'s binary and env, `settings` and `actor`,
/// as `mr status` reads it, or Unknown with joined_unknown_reason of the report when that read fails.
pub fn read_folder_holds(
    git: &Git,
    folder: &FolderDisk,
    settings: &SnapshotSettings,
    actor: &str,
) -> Result<FolderHolds, Report> {
    let unsaved = folder_unsaved(git, &folder.path).map_err(|error| folders_error_report(VERB, &error))?;
    let joined = match folder.record.kind {
        FolderKind::Zone => JoinedAnswer::ZoneFolder,
        FolderKind::Request | FolderKind::Attempt(_) => {
            let in_folder = Git { repo: folder.path.clone(), ..git.clone() };
            match read_branch_reads(&in_folder, &folder.path, &folder.record, settings, actor, VERB) {
                Ok(reads) if joined_from_reads(&folder.record.base, &reads) => JoinedAnswer::Joined,
                Ok(_) => JoinedAnswer::NotJoined,
                Err(refusal) => JoinedAnswer::Unknown(joined_unknown_reason(&refusal)),
            }
        }
    };
    Ok(FolderHolds { unsaved, joined })
}
