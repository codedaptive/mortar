//! run_folder_zone: `mr folder zone <zone> <path> [--start <commit>]`, a zone's standing folder made or refilled.
//! No sibling test (S-104): it takes a real lock and runs git, the op log and folders; folder_setup_for_tests and
//! folder_made_report_tests cover its decisions. The RIT crates/cli/tests/cli_rit/run_folder_zone_tests.rs covers it through
//! the binary: a sparse folder holding the zone's paths and root files, recorded as a zone folder, and its repeat;
//! --start making a new zone branch by an op of the actor; a --start that names no commit refused, nothing made.
//! RITs left for Nagatha (cross-zone, listed only, not written): a folders lock held by a live pid, exit 3; free disk
//! under the floor, exit 1 with the cli's facts; the zone branch checked out elsewhere or a different folder at the
//! path, the cli's why and next.

use std::path::Path;

use folders::make_zone_folder;
use gitio::Git;
use oplog::{KillPidProbe, LockName};

use super::{folder_error_report, folder_made_report, folder_setup_for};
use crate::config::Config;
use crate::context::{repository_error, repository_report};
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, ExitCode, Report};
use crate::shell::ShellContext;

const VERB: &str = "folder";

/// `path` is taken from `cwd` when relative. `start` is resolved to a commit before any lock (none: exit 1, nothing
/// made). Under join-<zone> after recover (locked_act), keyed by --key or op_key("folder-zone", the folder): folders'
/// make_zone_folder with `paths` (the zone's paths and root files) and the setup folder_setup_for gives for the actor.
pub fn run_folder_zone(
    context: &ShellContext,
    config: &Config,
    cwd: &Path,
    zone: &str,
    path: &Path,
    start: Option<&str>,
    paths: &[String],
) -> Report {
    let git = Git::new(cwd);
    let common = match git.common_dir() {
        Ok(common) => common,
        Err(error) => return repository_report(VERB, &repository_error(cwd, error)),
    };
    let start = match start.map(|rev| (rev, git.rev_parse(rev))) {
        None => None,
        Some((_, Ok(Some(commit)))) => Some(commit),
        Some((rev, Ok(None))) => return no_commit(rev),
        Some((_, Err(error))) => return failure_report(VERB, &error.to_string()),
    };
    let path = cwd.join(path);
    let key = op_key(context.args.key.as_deref(), "folder-zone", &path, &[]);
    let lock = LockName::Join(zone.to_string());
    locked_act(&git, &common, &lock, &key, &KillPidProbe, VERB, || {
        let setup = folder_setup_for(&git, path.clone(), config, &context.actor);
        let record =
            make_zone_folder(&setup, zone, paths, start.as_ref()).map_err(|error| folder_error_report("zone", &error))?;
        Ok(folder_made_report("zone", &path, &record))
    })
    .unwrap_or_else(|report| report)
}

/// Exit 1 "refused": `rev` names no commit in the repository; nothing was made.
fn no_commit(rev: &str) -> Report {
    Report {
        verb: VERB.to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![format!("--start {rev} names no commit in this repository"), "nothing was made".to_string()],
        owner: None,
        next: "repeat with a --start that names a commit".to_string(),
        fields: Vec::new(),
    }
}
