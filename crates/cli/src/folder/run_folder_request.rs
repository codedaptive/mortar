//! run_folder_request: `mr folder request <zone> <request> <path> --paths <path>...`, a request's folder made from
//! its zone's branch, holding only its paths and the root files.
//! No sibling test: it takes a real lock and runs git, the op log and folders; request_folder_paths_tests,
//! folder_setup_for_tests and folder_made_report_tests cover its decisions. The RIT
//! crates/cli/tests/cli_rit/run_folder_request_tests.rs covers it through the binary: the folder holding only its --paths and
//! the root file, recorded as a request folder, its branch made by an op of the actor; a zone with no branch refused,
//! nothing made.
//! Repo integration tests to write (cross-zone, not written here): a request branch already at another commit or
//! checked out elsewhere, the cli's why and next; a folders lock held by a live pid, exit 3; free disk under the floor, exit 1
//! with the cli's facts.

use std::path::Path;

use folders::make_request_folder;
use gitio::Git;
use oplog::{KillPidProbe, LockName};

use super::{folder_error_report, folder_made_report, folder_setup_for};
use crate::config::Config;
use crate::context::{repository_error, repository_report};
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::Report;
use crate::shell::ShellContext;

const VERB: &str = "folder";

/// `path` is taken from `cwd` when relative. Under join-<zone> after recover (locked_act), keyed by --key or
/// op_key("folder-request", the folder): folders' make_request_folder with `paths` (the --paths, then the root files)
/// and the setup folder_setup_for gives for the actor.
pub fn run_folder_request(
    context: &ShellContext,
    config: &Config,
    cwd: &Path,
    zone: &str,
    request: &str,
    path: &Path,
    paths: &[String],
) -> Report {
    let git = Git::new(cwd);
    let common = match git.common_dir() {
        Ok(common) => common,
        Err(error) => return repository_report(VERB, &repository_error(cwd, error)),
    };
    let path = cwd.join(path);
    let key = op_key(context.args.key.as_deref(), "folder-request", &path, &[]);
    let lock = LockName::Join(zone.to_string());
    locked_act(&git, &common, &lock, &key, &KillPidProbe, VERB, || {
        let setup = folder_setup_for(&git, path.clone(), config, &context.actor);
        let record = make_request_folder(&setup, zone, request, paths)
            .map_err(|error| folder_error_report("request", &error))?;
        Ok(folder_made_report("request", &path, &record))
    })
    .unwrap_or_else(|report| report)
}
