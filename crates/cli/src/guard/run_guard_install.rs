//! run_guard_install: `mr guard install`, guard.json and both guard hooks put in place for the config's branches,
//! with a warning line when the first `mr` on PATH is not this Mortar.
//! No sibling test: it takes a real lock, runs git, the op log and land, and reads PATH, this binary's path and
//! the file system; guard_in_place_tests, guard_installed_report_tests, guard_error_report_tests,
//! hooks_path_report_tests (the core.hooksPath refusal), install_outcome_tests (the warning on installed outcomes only,
//! last before next), with_path_warning_tests and path's path_warning_for_tests (the warning's decision),
//! same_build_tests, marker_in_chunks_tests, path_entries_tests and mr_path_warning_tests cover its decisions. The RIT
//! crates/cli/tests/cli_rit/run_guard_install_tests.rs covers it through the binary: a fresh repository gets guard.json
//! and both hooks, executable, each the guard's script (exit 0 "installed"); a repeat is "already-installed" and
//! changes no byte; a foreign pre-push is refused exit 1 "foreign-hook" with nothing written; without --actor, exit 4;
//! from a linked worktree the hooks land in the main repository's hooks folder; a relative core.hooksPath is refused
//! exit 1 "refused" with nothing written.
//! The validation test crates/cli/tests/validation/guard_linked_worktree.rs covers the refusal after install through
//! the binary: with an absolute core.hooksPath, `mr guard install` from one linked worktree writes the hooks there,
//! and in a second linked worktree made afterwards git update-ref of integration is refused through the installed
//! reference-transaction hook (which runs `mortar guard-hook`) with the guard's why, owner and next, while an
//! unprotected ref moves.
//! Repo integration tests to write (cross-zone, not written here): the land lock held by a live pid, exit
//! 3 (oplog's locks); GuardError::Git in the cli's own words; after `mr guard install`, git commit and push against
//! integration and origin are refused through the installed hook and Mortar's own moves pass (land's guard_decide);
//! tests/operational_rit/guard_fixture.rs to install through `mr guard install` instead of land::guard_install;
//! through the binary, a foreign `mr` first on a scratch PATH makes `mr guard install` exit 0 "installed" with the
//! PATH warning as the last line before next, and this build's bin dir first on PATH gives no warning; a copied
//! Mortar binary, not this path, first on PATH gives no warning; a myrepos-like `mr` beside Mortar gives the warning
//! as the last line before next, exit 0 "installed"; the PATH lookup (first_on_path) and the marker read
//! (file_has_marker) on real files through `mr guard install`, by the warning line present or absent.

use std::path::Path;

use gitio::Git;
use land::{check_hooks_path, guard_install, guard_status};
use oplog::{KillPidProbe, LockName};

use super::{
    guard_error_report, guard_in_place, guard_installed_report, guard_refs_for, hooks_path_report, install_outcome,
};
use crate::config::Config;
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::path::{file_has_marker, first_on_path, path_warning_for, MORTAR_MARKER};
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "guard";

/// First, before the lock, land's check_hooks_path: a core.hooksPath git would resolve per worktree, or one git could
/// not read, returns hooks_path_report with no lock taken, no recover run, nothing written and no PATH warning.
/// Then, under the land lock after recover (locked_act), keyed by --key or op_key("guard-install", the common dir):
/// whether the guard was already in place for the config's branches (a guard_status that fails counts as not), then
/// land's guard_install (refusing a foreign hook before writing anything), then guard_status read back for the report.
/// The result goes through install_outcome: the installed report (either outcome) gets path_warning_for's line, over
/// this binary's path, the first `mr` on PATH as the shell in `cwd` would find it (an unset PATH finds none), the
/// file system's canonicalize and file_has_marker's read of the candidate for MORTAR_MARKER, as its last;
/// refusals, failures and a busy lock do not.
/// Records no op: it moves no ref.
pub fn run_guard_install(context: &ShellContext, config: &Config, cwd: &Path) -> Report {
    let git = Git::new(cwd);
    let common = match git.common_dir() {
        Ok(common) => common,
        Err(_) => return failure_report(VERB, "git could not name the repository's common dir"),
    };
    if let Err(error) = check_hooks_path(&git) {
        return hooks_path_report(&error);
    }
    let refs = guard_refs_for(config);
    let key = op_key(context.args.key.as_deref(), "guard-install", &common, &[]);
    let installed = locked_act(&git, &common, &LockName::Land, &key, &KillPidProbe, VERB, || {
        let already = guard_status(&git).is_ok_and(|state| guard_in_place(&state, &refs));
        guard_install(&git, &refs).map_err(|error| guard_error_report(&error))?;
        let state = guard_status(&git).map_err(|error| guard_error_report(&error))?;
        Ok(guard_installed_report(already, &state))
    });
    let warning = path_warning_for(
        std::env::current_exe().ok().as_deref(),
        first_on_path("mr", std::env::var_os("PATH").as_deref(), cwd).as_deref(),
        &|path| path.canonicalize().ok(),
        &|path| file_has_marker(path, &MORTAR_MARKER),
    );
    install_outcome(installed, warning)
}
