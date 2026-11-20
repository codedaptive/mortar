//! run_folder_zone: `mr folder zone <zone> <path> [--start <commit>]`, a zone's standing folder made or refilled.
//! No sibling test: it takes a real lock and runs git, the op log and folders; folder_setup_for_tests and
//! folder_made_report_tests cover its decisions. The integration test crates/cli/tests/folder_runs/run_folder_zone_tests.rs covers it through
//! the binary: a sparse folder holding the zone's paths and root files, recorded as a zone folder, and its repeat;
//! --start making a new zone branch by an op of the actor; a --start that names no commit refused, nothing made.
//! With no --start, a missing zone branch is started at the origin branch tip (default_zone_start_tests covers
//! the decision); the same integration test file covers a bare folder zone on a missing zone branch made at the
//! origin branch tip, and, with no origin branch either, refused with a next naming --start.
//! Integration tests to write (not written here): a folders lock held by a live pid, exit 3; free disk
//! under the floor, exit 1 with the cli's facts; the zone branch checked out elsewhere or a different folder at the
//! path, the cli's why and next.

use std::path::Path;

use folders::make_zone_folder;
use folders::plan::zone_branch;
use gitio::{Git, Oid};
use oplog::{KillPidProbe, LockName};

use super::{default_zone_start, folder_error_report, folder_made_report, folder_setup_for};
use crate::config::Config;
use crate::context::{repository_error, repository_report};
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, ExitCode, Report};
use crate::shell::ShellContext;

const VERB: &str = "folder";

/// `path` is taken from `cwd` when relative. `start` is resolved to a commit before any lock (none: exit 1, nothing
/// made). With no `start` and no zone branch, the start is the tip of `config.origin_branch`; with no origin branch
/// either, there is none and make_zone_folder's refusal stands. Under join-<zone> after recover (locked_act), keyed by
/// --key or op_key("folder-zone", the folder): folders' make_zone_folder with `paths` (the zone's paths and root
/// files) and the setup folder_setup_for gives for the actor.
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
        None => match origin_start(&git, zone, &config.origin_branch) {
            Ok(commit) => commit,
            Err(report) => return report,
        },
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

/// The commit a bare `mr folder zone` starts `zone`'s branch at: the origin branch tip when the zone branch is
/// missing (default_zone_start), else None. A zone name zone_branch rejects gives None, for make_zone_folder to refuse.
fn origin_start(git: &Git, zone: &str, origin_branch: &str) -> Result<Option<Oid>, Report> {
    let failure = |error: gitio::GitError| failure_report(VERB, &error.to_string());
    let Ok(branch) = zone_branch(zone) else { return Ok(None) };
    let exists = git.read_ref(&format!("refs/heads/{branch}")).map_err(failure)?.is_some();
    default_zone_start(None, exists, origin_branch).map_or(Ok(None), |rev| git.rev_parse(&rev).map_err(failure))
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
