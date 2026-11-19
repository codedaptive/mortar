//! One command line, from the words to the report the binary prints: the line is parsed, the installed git is checked
//! against gitio's floor, and the verb run. The body is shell/run_in.rs; run gives it the real reads: git's version
//! from `cwd` and read_folder_context. Help and --version are never asked here: entry::main_entry answers every line
//! that asks for them before run is called. The floor check comes first, before any folder or config read, for every
//! verb but help, install, uninstall and guard-hook (shell/checks_git_floor.rs says why).
//! No unit test: run only wires real git into run_in. shell/run_in_tests and shell/git_floor_tests cover the floor
//! check and its order before the folder read with both reads faked; the tests in src (context/context_report_tests,
//! args/usage_report_tests, actor_for_tests, repository_report_tests, missing_config_report_tests,
//! shell/checks_git_floor_tests, shell/git_floor_report_tests) cover what it reports. The validation tests are to
//! cover, with the mortar binary as a child process in a throwaway repository: an unknown verb is exit 4
//! listing the verbs; `mr init` without
//! --actor is exit 4; with --json stdout holds one JSON object and nothing else, also for a usage error; an agent verb
//! outside a Mortar folder is refused, exit 1 (with `mr join`); a verb that needs the config is refused without
//! one, exit 1, next "mr init", and outside any git repository is refused, exit 1, outcome "not-a-folder" for an
//! agent verb (`mr join`) and "not-a-repository" for a harness verb that needs the config (with the first such
//! verb); `mortar guard-hook` runs in any repository with no folder and no --actor, and --actor with it is exit 4;
//! `mortar install -y` then `mortar uninstall`, with HOME a throwaway folder and outside any git repository, are
//! exit 0 and leave HOME as it was; install or uninstall with --actor is exit 4; every report names its verb ("" when
//! none was read).

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
