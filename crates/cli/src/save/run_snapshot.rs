//! run_snapshot: `mr snapshot`, the harness's end-of-turn and before-gate call (SPEC-cli 05): the folder it runs
//! in saved to its snapshot ref (refs/mortar/snap/<zone>/<request>, or refs/mortar/trysnap/<zone>/<request>/<k> in
//! attempt folder k: S-69), after the configured formatter, as save runs it; exit 0 also when nothing changed.
//! No sibling test (S-104): it takes a real lock and runs git, the formatter and the op log; snapshot_target_for,
//! snapshot_settings_for, op_key, zone_folder_refusal, snapshot_report and snapshot_error_report are its decisions
//! and have their tests.
//! RITs left for Nagatha (cross-zone, listed only, not written: S-135, S-136), with the mortar binary as a child
//! process in folders made by the landing verbs in a throwaway repository:
//! - snapshot in a sparse request folder writes the request's snap ref, exit 0, outcome snapshotted;
//! - snapshot with nothing changed since the last one is exit 0, outcome unchanged, and moves no ref;
//! - snapshot in a zone folder is refused, exit 1, naming the folder, and moves no ref;
//! - snapshot in an attempt folder moves its trysnap ref, not the request's snap ref;
//! - any argument is exit 4; without a config, exit 1; run outside a Mortar folder, exit 1;
//! - a join-<zone> lock held by a live pid is exit 3.
//!   ACCEPT-5 (05-disk.md): snapshot's run in a request folder after a small edit, reached through this verb.

use std::path::Path;

use gitio::Git;
use oplog::KillPidProbe;
use snapshot::snapshot_formatted;

use super::{snapshot_error_report, snapshot_report, snapshot_target_for, zone_folder_refusal};
use crate::args::no_arguments_report;
use crate::config::snapshot_settings_for;
use crate::context::{context_report, read_folder_context};
use crate::lock::{lock_name_for, locked_act};
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "snapshot";

/// Any argument is snapshot's usage report, before any git. Then the config, then the Mortar folder holding `cwd`
/// (read_folder_context; context_report when it cannot be read). A zone folder has no snapshot ref:
/// zone_folder_refusal, exit 1.
/// Under `join-<zone>` (S-84, from the verb table through lock_name_for) after recover (locked_act, S-16), keyed by
/// --key or op_key("snapshot", the folder) as the lock's holder only (S-132: snapshot_formatted draws its own op id),
/// snapshot_formatted with the config's excludes, cap and formatter; reported by snapshot_report, or by
/// snapshot_error_report when it did not happen.
pub fn run_snapshot(context: &ShellContext, cwd: &Path) -> Report {
    let args = &context.args;
    if let Some(usage) = no_arguments_report(VERB, &args.rest, args.json) {
        return usage;
    }
    let Some(config) = &context.config else {
        return failure_report(VERB, "no config for a verb that needs it; the shell loads it before snapshot runs");
    };
    let folder = match read_folder_context(cwd) {
        Ok(folder) => folder,
        Err(error) => return context_report(VERB, &error),
    };
    let Some(target) = snapshot_target_for(&folder.record, &context.actor) else {
        return zone_folder_refusal(&folder.toplevel);
    };
    let settings = snapshot_settings_for(config);
    let git = Git::new(&folder.toplevel);
    let key = op_key(args.key.as_deref(), VERB, &folder.toplevel, &[]);
    let Some(lock) = lock_name_for(args.verb.lock, &folder.record.zone) else {
        return failure_report(VERB, "the verb table gives snapshot no lock; it moves a ref and needs join-<zone>");
    };
    locked_act(&git, &folder.common, &lock, &key, &KillPidProbe, VERB, || {
        snapshot_formatted(
            &git,
            &folder.toplevel,
            &target,
            &settings.excludes,
            settings.cap_bytes,
            settings.formatter.as_ref(),
        )
        .map(|outcome| snapshot_report(&outcome))
        .map_err(|error| snapshot_error_report(&error))
    })
    .unwrap_or_else(|report| report)
}
