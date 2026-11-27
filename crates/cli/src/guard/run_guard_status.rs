//! run_guard_status: `mr guard status`, whether guard.json and both guard hooks are in place for the config's
//! branches, read without writing.
//! No sibling test: it takes a real lock and runs git, the op log and land; guard_state_report_tests and
//! guard_error_report_tests cover its decisions. The integration test crates/cli/tests/guard_runs/run_guard_status_tests.rs covers it through
//! the binary: an untouched repository is exit 0 "not-installed" with nothing written; after `mr guard install`
//! exit 0 "installed" naming the three branches; a foreign reference-transaction exit 3 "foreign-hook" naming its
//! path; a guard.json that is not a guard config exit 3 "refused" naming the file; a guard.json for other branches
//! than the config exit 0 "not-installed" naming both.
//! Integration tests to write (not written here): the land lock held by a live pid, exit
//! 4 (oplog's locks); GuardError::Git in the cli's own words.

use std::path::Path;

use gitio::Git;
use land::guard_status;
use oplog::{KillPidProbe, LockName};

use super::{guard_error_report, guard_refs_for, guard_state_report};
use crate::config::Config;
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "guard";

/// Under the land lock after recover (locked_act), keyed by --key or op_key("guard-status", the common dir): land's
/// guard_status, reported against the config's branches (guard_state_report). Writes nothing of the guard's.
pub fn run_guard_status(context: &ShellContext, config: &Config, cwd: &Path) -> Report {
    let git = Git::new(cwd);
    let common = match git.common_dir() {
        Ok(common) => common,
        Err(_) => return failure_report(VERB, "git could not name the repository's common dir"),
    };
    let key = op_key(context.args.key.as_deref(), "guard-status", &common, &[]);
    locked_act(&git, &common, &LockName::Land, &key, &KillPidProbe, VERB, || {
        let state = guard_status(&git).map_err(|error| guard_error_report(&error))?;
        Ok(guard_state_report(&state, &guard_refs_for(config)))
    })
    .unwrap_or_else(|report| report)
}
