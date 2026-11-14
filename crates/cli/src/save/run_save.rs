//! run_save: `mortar save "<sentence>"`, the agent verb that commits its folder's files on the request's branch.
//! No unit test: it takes the lock, runs git, the formatter and the op log (S-79); save_sentence, snapshot_target_for,
//! save_report, save_error_report, snapshot_settings_for and op_key are its decisions and have their tests.
//! Nagatha's integration group is to cover, with the mortar binary as a child process in a request folder made by
//! the landing verbs in a throwaway repository:
//! - a first save writes a commit with the sentence as subject and prints next mortar join, exit 0, outcome saved;
//! - a save in attempt folder k moves refs/mortar/trysnap/<zone>/<request>/<k>, not the request's snapshot ref;
//! - a repeat with nothing changed is exit 0, outcome unchanged, and writes no commit and moves no branch;
//! - a blank or two-line sentence is exit 1 and nothing runs (no lock file, no snapshot ref move);
//! - a new file over the config's cap_bytes is listed with its size and the cap, exit 0;
//! - with a formatter configured the saved tree holds the formatted file;
//! - a formatter past its timeout is listed and the save goes ahead;
//! - a held join-<zone> lock is exit 3 naming the holder;
//! - run in a zone folder it is refused, exit 1;
//! - without a config it is refused, exit 1, next mortar init;
//! - --json gives one object with commit, change_id, written, refused and cap.

use gitio::Git;
use oplog::KillPidProbe;
use snapshot::snapshot_formatted;

use super::{save_error_report, save_report, save_sentence, snapshot_target_for};
use crate::config::snapshot_settings_for;
use crate::lock::{lock_name_for, locked_act};
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

/// Checks the sentence before any lock, formatter or git; then, under `join-<zone>` (S-84) with recover run
/// first (S-16), runs the configured formatter through snapshot_formatted and commits with snapshot::save, both
/// with the config's excludes and cap. The key (--key, else the verb and the folder with no moves, S-84) names the
/// lock's holder only and is not passed to oplog's apply (S-90); a plain repeat is safe because save writes no
/// commit and moves no branch when the folder equals HEAD (S-34, S-41). The lock comes
/// from the verb table's LockNeed (lock_name_for). The shell gives an agent verb its folder (actor_for
/// refuses a zone folder, so the record names a request) and a verb that needs the config its config; a missing
/// one is a failure here.
pub fn run_save(context: &ShellContext) -> Report {
    let sentence = match save_sentence(&context.args.rest) {
        Ok(sentence) => sentence,
        Err(report) => return report,
    };
    let Some(folder) = &context.folder else {
        return failure_report("save", "no folder context for an agent verb; the shell reads it before save runs");
    };
    let Some(target) = snapshot_target_for(&folder.record, &context.actor) else {
        return failure_report("save", "the folder record names no request; actor_for refuses a zone folder");
    };
    let Some(config) = &context.config else {
        return failure_report("save", "no config for a verb that needs it; the shell loads it before save runs");
    };
    let settings = snapshot_settings_for(config);
    let zone = &folder.record.zone;
    let git = Git::new(&folder.toplevel);
    let key = op_key(context.args.key.as_deref(), "save", &folder.toplevel, &[]);
    let Some(lock) = lock_name_for(context.args.verb.lock, zone) else {
        return failure_report("save", "the verb table gives save no lock; save moves a ref and needs join-<zone>");
    };
    locked_act(&git, &folder.common, &lock, &key, &KillPidProbe, "save", || {
        let formatted = match &settings.formatter {
            Some(formatter) => snapshot_formatted(
                &git,
                &folder.toplevel,
                &target,
                &settings.excludes,
                settings.cap_bytes,
                Some(formatter),
            )
            .map_err(|error| save_error_report(&error))?
            .formatted,
            None => None,
        };
        let outcome =
            snapshot::save(&git, &folder.toplevel, &target, sentence, &settings.excludes, settings.cap_bytes)
                .map_err(|error| save_error_report(&error))?;
        Ok(save_report(&outcome, formatted.as_ref()))
    })
    .unwrap_or_else(|report| report)
}
