//! run_disk: `mr disk`, what each Mortar folder, the object store and the refs cost on disk and what a plain
//! mr sweep frees (SPEC-cli 05). Takes no lock and writes nothing (S-16).
//! No sibling test (S-104): it runs git and reads the file system through folders' disk; config_zones_tests,
//! no_zones_report_tests, folders_error_report_tests and disk_report_tests cover its decisions.
//! RITs left for Nagatha (cross-zone, listed only, not written: S-135, S-136): disk on a zone folder and a request
//! folder lists both with their files and build bytes and counts the object store once; disk with a config of no
//! zones refused, exit 1, nothing read; any argument, exit 4; without a config, exit 1. ACCEPT-5 (05-disk.md):
//! disk's per-folder bytes and its store-once figure, reached through this verb.

use std::path::Path;

use gitio::Git;

use super::disk_report;
use crate::args::no_arguments_report;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;
use crate::upkeep::{config_zones, folders_error_report, no_zones_report};

const VERB: &str = "disk";

/// The idle days handed to folders' disk. A plain `mr sweep` (no --idle-days) clears no build dir, and no
/// document sets a default, so disk's sweep figure counts no build dir: u64::MAX days is never reached (only a folder
/// with no files at all counts as idle, and its build bytes are 0), and the figure is what that plain sweep frees,
/// the refs part. The build bytes are shown per folder instead.
const NO_BUILD_SWEEP: u64 = u64::MAX;

/// Any argument is disk's usage report, before any git. Then the config's zones (config_zones; none is
/// no_zones_report, exit 1), then folders' disk over every zone with no extra ref and the context's protected refs,
/// reported by disk_report or, when folders refuses or fails, folders_error_report.
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
    match folders::disk(&Git::new(cwd), NO_BUILD_SWEEP, &zones, &[], &context.protected) {
        Ok(report) => disk_report(&report),
        Err(error) => folders_error_report(VERB, &error),
    }
}
