//! run_disk: `mr disk`, what each Mortar folder, the object store and the refs cost on disk, what each folder holds
//! (its unsaved counts and whether its request has joined, or why that is unknown) and what a plain mr sweep frees.
//! Takes no lock and writes nothing in the repository: the joined read builds the tree of each request and attempt
//! folder, and those objects go to an object quarantine (with_object_quarantine); its temp index and the quarantine dir
//! sit in the OS temp dir and are removed.
//! No sibling test: it runs git and reads the file system through folders' disk; config_zones_tests,
//! no_zones_report_tests, folders_error_report_tests, with_object_quarantine_tests and disk_report_tests cover its
//! decisions. The integration tests crates/cli/tests/across_kits/snapshot_overlap_disk_sweep/{disk,guards}.rs cover:
//! disk on a zone folder and a request folder lists both with their files and build bytes and counts the object store
//! once, the same over two runs; a request folder with one edited tracked file and two untracked files shows 1 and 2,
//! not joined, and leaves the repository and its object store as they were
//! (disk_holds_dirty_request_counts_without_writing); a clean saved and joined request shows 0, 0 and joined
//! (disk_holds_clean_saved_and_joined_request); a zone folder shows its counts and no joined answer
//! (disk_holds_zone_counts_without_a_joined_answer); a request folder whose branch is gone shows joined unknown with
//! the missing branch as its reason beside a healthy folder's answer, exit 0, writing nothing
//! (disk_answers_every_folder_when_one_joined_read_fails); disk with a config of no zones refused, exit 1, nothing
//! read; any argument, exit 4; without a config, exit 1. The acceptance run's disk step checks disk's per-folder bytes
//! and its store-once figure, reached through this verb.

use std::path::Path;

use gitio::Git;

use super::{disk_report, read_folder_holds, with_object_quarantine};
use crate::args::no_arguments_report;
use crate::config::snapshot_settings_for;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;
use crate::upkeep::{config_zones, folders_error_report, no_zones_report};

const VERB: &str = "disk";

/// The idle days handed to folders' disk. A plain `mr sweep` (no --idle-days) clears no build dir, and no
/// document sets a default, so disk's sweep figure counts no build dir: u64::MAX days is never reached (only a folder
/// with no files at all counts as idle, and its build bytes are 0), and the figure is what that plain sweep frees,
/// the refs part. The build bytes are shown per folder instead.
const NO_BUILD_SWEEP: u64 = u64::MAX;

/// Any argument is disk's usage report, before the verb runs git. Then the config's zones (config_zones; none is
/// no_zones_report, exit 1), then folders' disk over every zone with no extra ref and the context's protected refs,
/// reported by disk_report or, when folders refuses or fails, folders_error_report. With the report, each folder's
/// holds in order (read_folder_holds, with the config's snapshot settings and the context's actor), read inside one
/// object quarantine over the repository's objects (the common dir's, an absolute path). A joined read that fails
/// makes that folder's joined answer unknown and disk goes on; only a folder whose unsaved counts cannot be read
/// fails disk, the first with its report. A common dir or quarantine that cannot be had is failure_report.
pub fn run_disk(context: &ShellContext, cwd: &Path) -> Report {
    let args = &context.args;
    if let Some(usage) = no_arguments_report(VERB, &args.rest, args.json) {
        return usage;
    }
    let Some(config) = &context.config else {
        return failure_report(VERB, "no config for a verb that needs it; the shell loads it before disk runs");
    };
    let Some(zones) = config_zones(config) else {
        return no_zones_report(VERB);
    };
    let git = Git::new(cwd);
    let report = match folders::disk(&git, NO_BUILD_SWEEP, &zones, &[], &context.protected) {
        Ok(report) => report,
        Err(error) => return folders_error_report(VERB, &error),
    };
    let objects = match git.common_dir() {
        Ok(common) => common.join("objects"),
        Err(error) => return failure_report(VERB, &error.to_string()),
    };
    let settings = snapshot_settings_for(config);
    let holds = with_object_quarantine(&git, &objects, |quarantined| {
        report
            .folders
            .iter()
            .map(|folder| read_folder_holds(quarantined, folder, &settings, &context.actor))
            .collect::<Result<Vec<_>, Report>>()
    });
    match holds {
        Ok(Ok(holds)) => disk_report(&report, &holds),
        Ok(Err(failed)) => failed,
        Err(why) => failure_report(VERB, &why),
    }
}
