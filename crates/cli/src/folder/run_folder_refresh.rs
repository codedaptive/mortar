//! run_folder_refresh: `mr folder refresh <zone folder path>`, a zone folder's sparse paths set to its zone's paths
//! and the root files in the config, then the folder brought to its zone branch's tip by zone_folder_move's
//! bring_zone_folder.
//! No sibling test: it takes a real lock and runs git, the op log and folders; refresh_report_tests,
//! refresh_folder_refusal_tests, unknown_zone_report_tests and zone_folder_paths_tests cover its decisions. The
//! integration test crates/cli/tests/folder_runs/run_folder_refresh_tests.rs covers it through the binary: a zone
//! folder given relative to the cwd widened to the config's new paths and brought to the tip, then a repeat
//! already-refreshed with nothing changed; a request folder and an attempt folder refused, next mr join, nothing
//! changed; an unstaged edit in the way and a staged change matching no zone commit each refuse the move, the work
//! kept, the unstaged edit's next ending with the repeat; a plain directory and a plain git worktree each exit 3
//! not-a-folder; a zone folder whose zone the config no longer names exit 3 refused, next mr init --replace with the
//! zone; the join-<zone> lock held by a live pid exit 4 busy; each with the folder unchanged.

use std::path::Path;

use folders::set_zone_paths;
use gitio::Git;
use oplog::{read_op_log, KillPidProbe, LockName};

use super::{
    folder_error_report, read_folder_at, refresh_folder_refusal, refresh_report, unknown_zone_report, zone_folder_paths,
};
use crate::config::Config;
use crate::context::{repository_error, repository_report};
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, ExitCode, Report};
use crate::shell::ShellContext;
use crate::zone_folder_move::bring_zone_folder;

const VERB: &str = "folder";

/// `path` is taken from `cwd` when relative and read as the top of its worktree, the path the report and its repeat
/// name. Before any lock: its folder record by read_folder_at (none, or git finding no worktree there, is exit 3
/// "not-a-folder"); a request or attempt folder is refresh_folder_refusal's; a zone the config does not name is
/// unknown_zone_report.
/// Under join-<zone> after recover (locked_act), keyed by --key or op_key("folder-refresh", the folder): folders'
/// set_zone_paths with zone_folder_paths (its refusal is folder_error_report), the paths counting as changed when the
/// record's work-order or needed paths differ from those read before; the op log and refs/heads/zone/<zone>'s tip
/// read (a failure says the paths are set and the folder was not moved); bring_zone_folder; refresh_report.
pub fn run_folder_refresh(context: &ShellContext, config: &Config, cwd: &Path, path: &Path) -> Report {
    let git = Git::new(cwd);
    let common = match git.common_dir() {
        Ok(common) => common,
        Err(error) => return repository_report(VERB, &repository_error(cwd, error)),
    };
    let (folder, record) = match read_folder_at(&cwd.join(path), "refresh", not_a_folder) {
        Ok(read) => read,
        Err(report) => return report,
    };
    if let Some(report) = refresh_folder_refusal(&folder, &record) {
        return report;
    }
    let zone = record.zone.as_str();
    let Some(paths) = zone_folder_paths(config, zone) else {
        return unknown_zone_report(zone, config);
    };
    let key = op_key(context.args.key.as_deref(), "folder-refresh", &folder, &[]);
    locked_act(&git, &common, &LockName::Join(zone.to_string()), &key, &KillPidProbe, VERB, || {
        let set = set_zone_paths(&git, &folder, &paths, &KillPidProbe)
            .map_err(|error| folder_error_report("refresh", &error))?;
        let paths_changed =
            (&set.work_order_paths, &set.needed_paths) != (&record.work_order_paths, &record.needed_paths);
        let not_moved = |what: String| {
            failure_report(
                VERB,
                &format!("{what}; the zone folder {}'s paths are set and it was not moved", folder.display()),
            )
        };
        let records = read_op_log(&common).map_err(|_| not_moved("the op log could not be read".to_string()))?.records;
        let zone_ref = format!("refs/heads/zone/{zone}");
        let tip = match git.read_ref(&zone_ref) {
            Ok(Some(tip)) => tip,
            Ok(None) => return Err(not_moved(format!("{zone_ref} does not exist"))),
            Err(_) => return Err(not_moved(format!("git could not read {zone_ref}"))),
        };
        let outcome = bring_zone_folder(&git, &records, zone, &folder);
        Ok(refresh_report(zone, &folder, &tip, &paths, paths_changed, &outcome))
    })
    .unwrap_or_else(|report| report)
}

/// Exit 3 "not-a-folder": nothing at `path` is a folder Mortar made, for `reason`; nothing changed. mr folder zone
/// makes a zone folder.
fn not_a_folder(path: &Path, reason: &str) -> Report {
    Report {
        verb: VERB.to_string(),
        exit: ExitCode::Refused,
        outcome: "not-a-folder".to_string(),
        lines: vec![format!("{} is not a Mortar folder: {reason}", path.display()), "nothing changed".to_string()],
        owner: None,
        next: format!("mr folder zone <zone> {} --actor <name> makes a zone folder there", path.display()),
        fields: Vec::new(),
    }
}
