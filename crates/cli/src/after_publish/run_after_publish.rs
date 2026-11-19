//! run_after_publish: `mr after-publish <zone> [--move <zone>]...`, every named zone moved onto the origin a
//! publish left by land's after_publish under the land lock, then each named zone's folder brought to its zone
//! branch's tip. --key names the lock holder only, never passed to the op log.
//! After land's Ok answer (Moved, Current or Conflicted), each named zone's folder is brought to its tip from gitio
//! and the op log only (bring_zone_folder): no branch op, no saved check, nothing before land's call but reading the
//! zones' folders and open requests. A refused or failed folder never stops the other zones; a repeat answers
//! Current for each zone that Moved and brings the folders left behind.
//! No sibling test: it takes a real lock and runs git, the op log and land; parse_after_publish_args_tests,
//! find_zone_old_tests, zone_values_tests, zone_move_lines_tests, zone_folder_lines_tests,
//! after_publish_report_tests and after_publish_error_report_tests cover its decisions.
//! Validation tests to write (not written here): zone b's folder
//! made before r3 joined ends with r3's change; a folder several joins behind brought to the tip from the commit it
//! stands on; Current and Conflicted zones' folders brought to the tip; a folder left on a pre-replay tip by a refused
//! repeat found through the op log; no folder; already at the tip; an unstaged edit in the way and an unmerged index
//! each refuse that folder, the others still moved, exit 1, and clearing it as the next says then repeating
//! finishes; the staged-edit loss run (a staged edit at a path the tip changes: that folder refused, "matches no
//! commit", the edit kept, exit 1); open requests listed to rebase; SquashOfNotInZone refused naming the zone and the source;
//! no branch op written for a folder move; the land lock held by a live pid, exit 3; without a config, exit 1.

use std::path::Path;

use gitio::Git;
use land::{ZoneMove, ZoneRequests};
use oplog::{read_op_log, KillPidProbe, LockName};

use super::{
    after_publish_error_report, after_publish_failure, after_publish_report, bring_zone_folder, parse_after_publish_args,
    read_named_zones, ZoneFolderOutcome,
};
use crate::args::{usage_report, UsageError};
use crate::guard::guard_refs_for;
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "after-publish";

/// The config, then the words read by parse_after_publish_args (an Err is after-publish's usage report, exit 4,
/// before the verb runs git). Then, under the land lock after recover (locked_act), keyed by --key or
/// op_key("after-publish", the common dir): the named zones' folders and open requests (read_named_zones); land's
/// after_publish as the context's actor, the published zone first (a refusal is after_publish_error_report); the op
/// log read once; then for each ZoneMove in order, the folder of the named zone it names brought to the tip
/// (bring_zone_folder), NoFolder when it has none; reported by after_publish_report. An op log that cannot be read is
/// a failure saying the zones moved and their folders did not.
pub fn run_after_publish(context: &ShellContext, cwd: &Path) -> Report {
    let args = &context.args;
    let Some(config) = &context.config else {
        return failure_report(VERB, "no config for a verb that needs it; the shell loads it before after-publish runs");
    };
    let words = match parse_after_publish_args(&args.rest, args.dash_dash, config) {
        Ok(words) => words,
        Err(reason) => {
            return usage_report(&UsageError { verb: Some(VERB.to_string()), reason, json: args.json });
        }
    };
    let git = Git::new(cwd);
    let common = match git.common_dir() {
        Ok(common) => common,
        Err(_) => return failure_report(VERB, "git could not name the repository's common dir"),
    };
    let key = op_key(args.key.as_deref(), VERB, &common, &[]);
    locked_act(&git, &common, &LockName::Land, &key, &KillPidProbe, VERB, || {
        let zones: Vec<String> = std::iter::once(&words.published).chain(&words.moves).cloned().collect();
        let named = read_named_zones(&git, &zones)?;
        let requests: Vec<ZoneRequests> = named
            .iter()
            .map(|zone| ZoneRequests { zone: zone.zone.clone(), requests: zone.requests.clone() })
            .collect();
        let (published, others) = requests.split_at(1);
        let moves = land::after_publish(&git, &guard_refs_for(config), &published[0], others, &context.actor)
            .map_err(|error| after_publish_error_report(&error, &config.integration_branch))?;
        let records = read_op_log(&common)
            .map_err(|_| {
                after_publish_failure(vec![
                    "the op log could not be read; the zones are moved and their folders are not".to_string()
                ])
            })?
            .records;
        let zones: Vec<_> = moves
            .into_iter()
            .map(|zone_move| {
                let name = zone_of(&zone_move);
                let folder = named.iter().find(|zone| zone.zone == name).and_then(|zone| zone.folder.as_ref());
                let outcome = match folder {
                    Some(folder) => bring_zone_folder(&git, &records, name, folder),
                    None => ZoneFolderOutcome::NoFolder,
                };
                (zone_move, outcome)
            })
            .collect();
        Ok(after_publish_report(&words, &zones))
    })
    .unwrap_or_else(|report| report)
}

/// The zone a ZoneMove names, arm by arm: its folder is looked up by this name, never by order.
fn zone_of(zone_move: &ZoneMove) -> &str {
    match zone_move {
        ZoneMove::Moved { zone, .. } => zone,
        ZoneMove::Current { zone, .. } => zone,
        ZoneMove::Conflicted { zone, .. } => zone,
    }
}
