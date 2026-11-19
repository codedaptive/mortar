//! run_folder: `mr folder <sub-verb> ...`, the harness verb that makes, removes, multiplies and settles Mortar
//! folders: its words read, a zone the config does not name refused, then the sub-verb's runner.
//! No sibling test: it reaches git, the op log, folders and snapshot through its runners;
//! parse_folder_args_tests, zone_folder_paths_tests and request_folder_paths_tests cover its decisions. The RITs
//! crates/cli/tests/cli_rit/run_folder_{zone,request,remove,attempts,keep,drop}_tests.rs cover it through the binary: usage
//! errors (no --paths, n = 0) exit 4 with folder's usage line; a zone not in the config exit 1, nothing made, next
//! naming `mr init --replace` with `--zone <zone>=`.
//! Repo integration tests to write (cross-zone, not written here): run outside any git repository it is refused exit 1
//! (the shell's repository_report); without a config, exit 1, next mr init.

use std::path::Path;

use super::{
    parse_folder_args, request_folder_paths, run_folder_attempts, run_folder_drop, run_folder_keep, run_folder_remove,
    run_folder_request, run_folder_zone, zone_folder_paths, FolderCommand,
};
use crate::args::{usage_report, UsageError};
use crate::config::Config;
use crate::report::{failure_report, ExitCode, Report};
use crate::shell::ShellContext;

const VERB: &str = "folder";

/// The words after the verb read by parse_folder_args (an Err is folder's usage report, exit 4, before any git).
/// zone and request name a zone that must be in the config (else exit 1 "refused", nothing made); zone's paths are
/// zone_folder_paths', request's its --paths then the root files (request_folder_paths). Then the sub-verb's runner.
pub fn run_folder(context: &ShellContext, cwd: &Path) -> Report {
    let args = &context.args;
    let command = match parse_folder_args(&args.rest) {
        Ok(command) => command,
        Err(reason) => {
            return usage_report(&UsageError { verb: Some(VERB.to_string()), reason, json: args.json });
        }
    };
    let Some(config) = &context.config else {
        return failure_report(VERB, "no config for a verb that needs it; the shell loads it before folder runs");
    };
    match command {
        FolderCommand::Zone { zone, path, start } => match zone_folder_paths(config, &zone) {
            Some(paths) => run_folder_zone(context, config, cwd, &zone, &path, start.as_deref(), &paths),
            None => unknown_zone(&zone, config),
        },
        FolderCommand::Request { zone, request, path, paths } => {
            if !config.zones.contains_key(&zone) {
                return unknown_zone(&zone, config);
            }
            let paths = request_folder_paths(&paths, config);
            run_folder_request(context, config, cwd, &zone, &request, &path, &paths)
        }
        FolderCommand::Remove { path } => run_folder_remove(context, config, cwd, &path),
        FolderCommand::Attempts { n, dir } => run_folder_attempts(context, config, cwd, n, &dir),
        FolderCommand::Keep { k } => run_folder_keep(context, config, cwd, k),
        FolderCommand::Drop => run_folder_drop(context, config, cwd),
    }
}

/// Exit 1 "refused": the zone is not in the config's zones (those it has are named), nothing was made; the next step
/// is the harness's own: init's whole command with --replace, the zone added to what the config already holds.
fn unknown_zone(zone: &str, config: &Config) -> Report {
    let known: Vec<&str> = config.zones.keys().map(String::as_str).collect();
    Report {
        verb: VERB.to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![
            format!("zone {zone} is not in the Mortar config, whose zones are: {}", known.join(", ")),
            "nothing was made".to_string(),
        ],
        owner: Some("the zones in the Mortar config".to_string()),
        next: format!(
            "mr init --replace --actor <name> --origin <branch> --integration <branch> --remote <name> \
             --zone {zone}=<path>[,<path>...] with every other zone and setting the config already holds, then repeat"
        ),
        fields: Vec::new(),
    }
}
