//! `mr init`: writes the repository's Mortar config.
//! No unit test: it runs git and reads and writes a real file; parse_init_args_tests, decide_init_tests,
//! init_report_tests and repository_report_tests cover its parts. Integration tests to write (not written here), with the
//! mortar binary as a child process in a throwaway repository: init writes <common>/mortar/config.json, exit 0 "written",
//! and it decodes to the values given; a repeat with the same values is exit 0 "already-done" and the file is
//! byte-identical; a repeat with different values is exit 3 naming the differing keys, the file unchanged; the same with
//! --replace is exit 0 "replaced" and the file holds the new values; init from a linked worktree writes the main
//! repository's common dir; init outside any git repository is refused, exit 3, outcome "not-a-repository", naming the
//! directory; a bad init argument is exit 2 with init's usage as next.

use std::path::Path;

use gitio::Git;

use super::{decide_init, init_report, parse_init_args, InitDecision};
use crate::args::{usage_report, ParsedArgs, UsageError};
use crate::config::{config_path, read_config, write_config};
use crate::context::{repository_error, repository_report};
use crate::report::{failure_report, Report};
use crate::shell::config_refusal;

const VERB: &str = "init";

/// Reads the options from `args.rest`, finds the common dir from `cwd`, decides against the config already
/// there and writes on Write or Replace. A config that cannot be read or decoded is refused, nothing written.
pub fn run_init(args: &ParsedArgs, cwd: &Path) -> Report {
    let options = match parse_init_args(&args.rest) {
        Ok(options) => options,
        Err(reason) => {
            return usage_report(&UsageError { verb: Some(VERB.to_string()), reason, json: args.json });
        }
    };
    let common = match Git::new(cwd).common_dir() {
        Ok(common) => common,
        Err(error) => return repository_report(VERB, &repository_error(cwd, error)),
    };
    let path = config_path(&common);
    let existing = match read_config(&common) {
        Ok(existing) => existing,
        Err(error) => return config_refusal(VERB, &error, &path),
    };
    let decision = decide_init(existing.as_ref(), &options.config, options.replace);
    if matches!(decision, InitDecision::Write | InitDecision::Replace { .. }) {
        if let Err(error) = write_config(&common, &options.config) {
            return failure_report(VERB, &error.reason);
        }
    }
    init_report(&decision, &path)
}
