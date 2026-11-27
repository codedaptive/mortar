//! One command line, from the words to the report the binary prints: the line is parsed, the installed git is checked
//! against gitio's floor, and the verb run. The body is shell/run_in.rs; run gives it the real reads: git's version
//! from `cwd` and read_folder_context. Help and --version are never asked here: entry::main_entry answers every line
//! that asks for them before run is called. The floor check comes first, before any folder or config read, for every
//! verb but help, install, uninstall and guard-hook (shell/checks_git_floor.rs says why).
//! No unit test: run only wires real git into run_in. shell/run_in_tests and shell/git_floor_tests cover the floor
//! check and its order before the folder read with both reads faked; the tests in src (context/context_report_tests,
//! args/usage_report_tests, actor_for_tests, repository_report_tests, missing_config_report_tests,
//! shell/checks_git_floor_tests, shell/git_floor_report_tests) cover what it reports. The integration test
//! crates/cli/tests/log_restore_runs/run_git_floor_tests.rs covers the real version read: with git 2.45.2 first on
//! PATH, `mr join` outside a folder is refused by the floor, exit 3; with 2.46.0 it passes the floor and is refused
//! as "not-a-folder" (mr_join_with_git_2_46_is_refused_as_not_a_folder runs it outside any git repository, so it also
//! covers an agent verb outside a Mortar folder, exit 3, outcome "not-a-folder"). The other cases are integration tests
//! to write (not written here, except where noted), with the mortar binary as a child process in a throwaway repository:
//! an unknown verb is exit 2 listing the verbs; `mr init` without --actor is exit 2; with --json stdout holds one JSON
//! object and nothing else, also for a usage error; a verb that needs the config is refused without one, exit 3, next "mr
//! init" (written: crates/cli/tests/save_join_runs/saves.rs, zone_save_and_missing_config_refuse_before_any_work), and
//! outside any git repository is refused, exit 3, outcome "not-a-repository" for a harness verb that needs the config
//! (with the first such verb); `mortar guard-hook` runs in any repository with no folder and no --actor, and --actor with
//! it is exit 2 (written: crates/cli/tests/guard_runs/guard.rs,
//! guard_allow_is_silent_in_plain_and_json_without_actor_or_folder); `mortar install -y` then `mortar uninstall`, with
//! HOME a throwaway folder and outside any git repository, are exit 0 and leave HOME as it was; install or uninstall with
//! --actor is exit 2; every report names its verb ("" when none was read).

use std::path::Path;

use gitio::Git;

use super::{run_in, ShellIo};
use crate::context::read_folder_context;
use crate::report::Report;

/// The report of `argv` (without the program name) run in `cwd`, and whether to render it as JSON: run_in with
/// `Git::new(cwd).version()` answering the floor check and read_folder_context reading the folder.
pub fn run(argv: &[String], cwd: &Path) -> (Report, bool) {
    run_in(argv, cwd, &ShellIo { read_version: &|| Git::new(cwd).version(), read_folder: &read_folder_context })
}
