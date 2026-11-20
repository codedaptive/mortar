//! run_folder_attempts: `mr folder attempts <n> <dir>`, run in a request folder: n attempt folders of the request
//! made at <dir>/try-<k>, each on try/<zone>/<request>/<k> at the request's tip.
//! A relative <dir> is taken from the request folder's toplevel, not from the subfolder it is run in.
//! No sibling test: it takes a real lock and runs git, the op log and folders; request_folder_refusal_tests,
//! attempts_dir_tests, folder_setup_for_tests and attempts_made_report_tests cover its decisions. The integration test
//! crates/cli/tests/folder_runs/run_folder_attempts_tests.rs covers it through the binary: attempts at <dir>/try-<k> with
//! Attempt(k) records and try branches at the request tip, made by ops of the actor; a relative <dir> run from a
//! subfolder taken from the request folder; run in a zone folder it is refused exit 1, nothing made; n = 0 or a <dir>
//! that normalizes inside the request folder is exit 4.
//! Integration tests to write (not written here): a folders lock held by a live pid, exit 3; free disk
//! under the floor at attempt j, exit 1 with the cli's facts, and a repeat making the rest; a request branch gone, the
//! cli's why and next.

use std::path::{Path, PathBuf};

use folders::make_attempts;
use folders::plan::attempt_path;
use folders::FolderRecord;
use gitio::Git;
use oplog::{KillPidProbe, LockName};

use super::{attempts_dir, attempts_made_report, folder_error_report, folder_setup_for, request_folder_refusal};
use crate::args::{usage_report, UsageError};
use crate::config::Config;
use crate::context::{context_report, read_folder_context};
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "folder";

/// `cwd` must be in a request folder (read_folder_context; a zone or attempt folder is request_folder_refusal's exit
/// 1). `dir` is resolved by attempts_dir: taken from that folder's toplevel when relative, normalized, and it must lie
/// outside that folder (the toplevel or inside it is folder's usage, exit 4).
/// Under join-<zone> after recover (locked_act), keyed by --key or op_key("folder-attempts", dir): folders'
/// make_attempts at `dir` on the request record's cone (its work order paths, then its needed paths), as the actor.
pub fn run_folder_attempts(context: &ShellContext, config: &Config, cwd: &Path, n: u32, dir: &Path) -> Report {
    let folder = match read_folder_context(cwd) {
        Ok(folder) => folder,
        Err(error) => return context_report(VERB, &error),
    };
    if let Some(refusal) = request_folder_refusal("folder attempts <n> <dir>", &folder.record) {
        return refusal;
    }
    let Some(request) = folder.record.request.as_deref() else {
        return failure_report(VERB, "the request folder's record names no request");
    };
    let dir = match attempts_dir(&folder.toplevel, dir) {
        Ok(dir) => dir,
        Err(reason) => {
            return usage_report(&UsageError { verb: Some(VERB.to_string()), reason, json: context.args.json })
        }
    };
    let zone = &folder.record.zone;
    let git = Git::new(&folder.toplevel);
    let paths: Vec<String> =
        folder.record.work_order_paths.iter().chain(&folder.record.needed_paths).cloned().collect();
    let key = op_key(context.args.key.as_deref(), "folder-attempts", &dir, &[]);
    let lock = LockName::Join(zone.clone());
    locked_act(&git, &folder.common, &lock, &key, &KillPidProbe, VERB, || {
        let setup = folder_setup_for(&git, dir.clone(), config, &context.actor);
        let records =
            make_attempts(&setup, zone, request, &paths, n).map_err(|error| folder_error_report("attempts", &error))?;
        // make_attempts answers attempts 1..=n in k order.
        let made: Vec<(PathBuf, FolderRecord)> =
            (1..).zip(records).map(|(k, record)| (attempt_path(&dir, k), record)).collect();
        Ok(attempts_made_report(&dir, &made))
    })
    .unwrap_or_else(|report| report)
}

