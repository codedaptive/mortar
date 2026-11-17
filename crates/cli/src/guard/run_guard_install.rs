//! run_guard_install: `mortar guard install`, guard.json and both guard hooks put in place for the config's branches.
//! No sibling test (S-104): it takes a real lock and runs git, the op log and land; guard_in_place_tests,
//! guard_installed_report_tests and guard_error_report_tests cover its decisions. The RIT
//! crates/cli/tests/cli_rit/run_guard_install_tests.rs covers it through the binary: a fresh repository gets guard.json and
//! both hooks, executable, each the guard's script (exit 0 "installed"); a repeat is "already-installed" and changes
//! no byte; a foreign pre-push is refused exit 1 "foreign-hook" with nothing written; without --actor, exit 4; from a
//! linked worktree the hooks land in the main repository's hooks folder; a relative core.hooksPath is honoured.
//! RITs left for Nagatha (cross-zone, listed only, not written: S-101, S-119): the land lock held by a live pid, exit
//! 3 (oplog's locks); GuardError::Git in the cli's own words; after `mortar guard install`, git update-ref, commit and
//! push against integration and origin are refused through the installed hook and Mortar's own moves pass (ACCEPT-2
//! checks 1-9; land's guard_decide); tests/operational_rit/guard_fixture.rs to install through `mortar guard install`
//! instead of land::guard_install.

use std::path::Path;

use gitio::Git;
use land::{guard_install, guard_status};
use oplog::{KillPidProbe, LockName};

use super::{guard_error_report, guard_in_place, guard_installed_report, guard_refs_for};
use crate::config::Config;
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "guard";

/// Under the land lock after recover (locked_act), keyed by --key or op_key("guard-install", the common dir): whether
/// the guard was already in place for the config's branches (a guard_status that fails counts as not), then land's
/// guard_install (refusing a foreign hook before writing anything), then guard_status read back for the report.
/// Records no op: it moves no ref.
pub fn run_guard_install(context: &ShellContext, config: &Config, cwd: &Path) -> Report {
    let git = Git::new(cwd);
    let common = match git.common_dir() {
        Ok(common) => common,
        Err(_) => return failure_report(VERB, "git could not name the repository's common dir"),
    };
    let refs = guard_refs_for(config);
    let key = op_key(context.args.key.as_deref(), "guard-install", &common, &[]);
    locked_act(&git, &common, &LockName::Land, &key, &KillPidProbe, VERB, || {
        let already = guard_status(&git).is_ok_and(|state| guard_in_place(&state, &refs));
        guard_install(&git, &refs).map_err(|error| guard_error_report(&error))?;
        let state = guard_status(&git).map_err(|error| guard_error_report(&error))?;
        Ok(guard_installed_report(already, &state))
    })
    .unwrap_or_else(|report| report)
}
