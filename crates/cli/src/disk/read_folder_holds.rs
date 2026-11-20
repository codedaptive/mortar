//! read_folder_holds: what one folder `mr disk` lists holds: its unsaved counts and whether its request has joined.
//! No unit test: it runs git; joined_from_reads_tests covers the joined rule, and the integration tests
//! crates/cli/tests/across_kits/snapshot_overlap_disk_sweep/disk.rs (disk_holds_dirty_request_counts_without_writing,
//! disk_holds_clean_saved_and_joined_request, disk_holds_zone_counts_without_a_joined_answer) cover what it reads.

use folders::{folder_unsaved, FolderDisk, FolderKind};
use gitio::Git;

use super::FolderHolds;
use crate::config::SnapshotSettings;
use crate::report::Report;
use crate::status::{joined_from_reads, read_branch_reads};
use crate::upkeep::folders_error_report;

const VERB: &str = "disk";

/// The folder's unsaved counts (folders::folder_unsaved; its Err is folders_error_report for disk) and, unless it
/// is a zone folder (joined None), whether its request has joined its zone: joined_from_reads over
/// read_branch_reads, run in the folder with `git`'s binary and env, `settings` and `actor`, as `mr status` reads
/// it. A failing joined read is its report, for disk.
pub fn read_folder_holds(
    git: &Git,
    folder: &FolderDisk,
    settings: &SnapshotSettings,
    actor: &str,
) -> Result<FolderHolds, Report> {
    let unsaved = folder_unsaved(git, &folder.path).map_err(|error| folders_error_report(VERB, &error))?;
    let joined = match folder.record.kind {
        FolderKind::Zone => None,
        FolderKind::Request | FolderKind::Attempt(_) => {
            let in_folder = Git { repo: folder.path.clone(), ..git.clone() };
            let reads = read_branch_reads(&in_folder, &folder.path, &folder.record, settings, actor, VERB)?;
            Some(joined_from_reads(&folder.record.base, &reads))
        }
    };
    Ok(FolderHolds { unsaved, joined })
}
