//! run_folder: `mr folder <sub-verb> ...`, the harness verb that makes, removes, refreshes, multiplies and settles
//! Mortar folders: its words read, a zone the config does not name refused, then the sub-verb's runner.
//! No sibling test: it reaches git, the op log, folders and snapshot through its runners;
//! parse_folder_args_tests, zone_folder_paths_tests, request_folder_paths_tests and unknown_zone_report_tests cover
//! its decisions. The integration tests
//! crates/cli/tests/folder_runs/run_folder_{zone,request,remove,refresh,attempts,keep,drop}_tests.rs cover it through
//! the binary: usage errors (no --paths, n = 0, refresh with no path) exit 2 with folder's usage line; a zone not in
//! the config exit 3, nothing made, next naming `mr init --replace` with `--zone <zone>=`. partner_cut_tests.rs
//! covers a --with partner not in the config: exit 3, nothing made.
//! Integration tests to write (not written here): run outside any git repository it is exit 2
//! outcome "not-a-repository" (the shell's repository_report); without a config, exit 2, next mr init.

use std::iter::once;
use std::path::Path;

use super::{
    parse_folder_args, request_folder_paths, run_folder_attempts, run_folder_drop, run_folder_keep,
    run_folder_refresh, run_folder_remove, run_folder_request, run_folder_zone, unknown_zone_report,
    zone_folder_paths, FolderCommand,
};
use crate::args::{usage_report, UsageError};
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "folder";

/// The words after the verb read by parse_folder_args (an Err is folder's usage report, exit 2, before the verb runs
/// git). zone and request name a zone that must be in the config (else exit 3 "refused", nothing made), and so must
/// each of request's --with partners, checked after the zone in the order given; zone's paths are
/// zone_folder_paths', request's its --paths then the root files (request_folder_paths); refresh reads its zone from
/// the folder's record. Then the sub-verb's runner.
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
            None => unknown_zone_report(&zone, config),
        },
        FolderCommand::Request { zone, request, path, paths, partners } => {
            if let Some(unknown) = once(&zone).chain(&partners).find(|zone| !config.zones.contains_key(*zone)) {
                return unknown_zone_report(unknown, config);
            }
            let paths = request_folder_paths(&paths, config);
            run_folder_request(context, config, cwd, &zone, &request, &path, &paths, &partners)
        }
        FolderCommand::Remove { path } => run_folder_remove(context, config, cwd, &path),
        FolderCommand::Refresh { path } => run_folder_refresh(context, config, cwd, &path),
        FolderCommand::Attempts { n, dir } => run_folder_attempts(context, config, cwd, n, &dir),
        FolderCommand::Keep { k } => run_folder_keep(context, config, cwd, k),
        FolderCommand::Drop => run_folder_drop(context, config, cwd),
    }
}
