//! `mortar join`: finishes a folder move a kill left, saves the folder's work, joins the request to its zone and
//! moves the folder after a rebase, all under the zone's join lock after recover (S-72, S-86, S-47, S-49).
//! No unit test: it runs git, the formatter and the op log, and takes a real lock (S-5); find_pending_move_tests,
//! snapshot_settings_for_tests, join_save_subject_tests, save_part_tests, join_report_tests, join_refusal_report_tests
//! and move_refusal_report_tests cover its decisions and reports. Nagatha's integration group is to cover, with the
//! mortar binary as a child process in a throwaway repository and a request folder: a request ahead of an unmoved zone
//! fast-forwards it, exit 0 "joined", next "wait for the zone's tests", and a repeat answers joined with nothing moved;
//! unsaved edits are saved first as "<request>: work saved at join" on the request branch and joined; a zone moved
//! under the request rebases it, exit 0 "rebased", the folder's files and HEAD at the new tip, next "mortar join"; a
//! conflicting zone change is exit 2 listing the files, no ref moved, next "mortar resolve"; a kill after join's rebase
//! op and before the folder move (the branch ahead of the folder) is finished by the next join before it saves, with
//! the finished move in its report; a local edit in the way of the folder move is exit 1 listing the paths, and once
//! moved aside the next join finishes the move; a missing zone branch is exit 1 owned by the zone's coordinator; any
//! argument is exit 4 with join's usage; a held join lock is exit 3; a configured formatter runs over the changed files
//! before the save.

use folders::move_folder;
use folders::plan::request_branch;
use gitio::Git;
use oplog::KillPidProbe;
use snapshot::{save, snapshot, snapshot_formatted, SaveOutcome, SnapshotError, SnapshotTarget};

use super::{
    join_refusal_report, join_report, join_save_subject, move_refusal_report, read_pending_move, JoinOutcome,
    PendingMove,
};
use crate::args::{usage_report, UsageError};
use crate::config::{snapshot_settings_for, SnapshotSettings};
use crate::context::FolderContext;
use crate::lock::{lock_name_for, locked_act};
use crate::opkey::op_key;
use crate::report::{failure_report, Report};
use crate::shell::ShellContext;

const VERB: &str = "join";

/// Takes no arguments. The op key is the verb and the folder with no moves (S-84(4)), unless --key gives one.
pub fn run_join(context: &ShellContext) -> Report {
    let args = &context.args;
    if !args.rest.is_empty() {
        let reason = "join takes no arguments".to_string();
        return usage_report(&UsageError { verb: Some(VERB.to_string()), reason, json: args.json });
    }
    let (Some(folder), Some(config)) = (&context.folder, &context.config) else {
        return failure_report(VERB, "join runs with its folder and the Mortar config, and the shell gave it neither");
    };
    let Some(request) = folder.record.request.as_deref() else {
        return failure_report(VERB, "this folder's record names no request");
    };
    let zone = folder.record.zone.as_str();
    let request_ref = match request_branch(zone, request) {
        Ok(branch) => format!("refs/heads/{branch}"),
        Err(error) => return failure_report(VERB, &error.to_string()),
    };
    let Some(lock) = lock_name_for(args.verb.lock, zone) else {
        return failure_report(VERB, "join's verb entry names no lock");
    };
    let (zone, request, actor) = (zone.to_string(), request.to_string(), context.actor.clone());
    let target = SnapshotTarget { zone, request, attempt: None, actor };
    let settings = snapshot_settings_for(config);
    let run = JoinRun { git: Git::new(&folder.toplevel), folder, settings, target, request_ref };
    let key = op_key(args.key.as_deref(), VERB, &folder.toplevel, &[]);
    match locked_act(&run.git, &folder.common, &lock, &key, &KillPidProbe, VERB, || run.act()) {
        Ok(report) | Err(report) => report,
    }
}

/// What one join reads once, before its lock.
struct JoinRun<'a> {
    git: Git,
    folder: &'a FolderContext,
    settings: SnapshotSettings,
    target: SnapshotTarget,
    request_ref: String,
}

impl JoinRun<'_> {
    /// Finish a pending folder move, save, join, and move the folder after a rebase (saved: the save just ran).
    fn act(&self) -> Result<Report, Report> {
        let finished_move = self.finish_pending_move()?;
        let save = self.save_work()?;
        let (folder, actor) = (&self.folder.toplevel, &self.target.actor);
        let answer = ::join::join(&self.git, folder, &self.target.zone, &self.request_ref, actor)
            .map_err(|error| join_refusal_report(&error))?;
        if let ::join::JoinAnswer::Rebased { from, to, .. } = &answer {
            move_folder(&self.git, folder, from, to, true, actor, None).map_err(|error| {
                let done = format!("rebased: {} moved from {from} to {to}, replayed onto the zone", self.request_ref);
                move_refusal_report(&error, vec![done])
            })?;
        }
        Ok(join_report(&JoinOutcome { request_ref: self.request_ref.clone(), finished_move, save, answer }))
    }

    /// The last rebase's folder move, if pending (S-86; a kill left it, or it is done and repeats as a no-op):
    /// snapshot the folder's work first (S-74), then move the folder from the op's old tip to its new one; saved is
    /// true from that snapshot.
    fn finish_pending_move(&self) -> Result<Option<PendingMove>, Report> {
        let Some(pending) = read_pending_move(&self.git, &self.folder.common, &self.request_ref, VERB)? else {
            return Ok(None);
        };
        let (excludes, cap) = (&self.settings.excludes, self.settings.cap_bytes);
        snapshot(&self.git, &self.folder.toplevel, &self.target, excludes, cap).map_err(snapshot_failure)?;
        move_folder(&self.git, &self.folder.toplevel, &pending.old, &pending.new, true, &self.target.actor, None)
            .map_err(|error| move_refusal_report(&error, Vec::new()))?;
        Ok(Some(pending))
    }

    /// Runs the formatter with a snapshot, then saves (S-72). Always run: save writes nothing when the folder's
    /// tree equals HEAD's.
    fn save_work(&self) -> Result<SaveOutcome, Report> {
        let (folder, excludes, cap) = (&self.folder.toplevel, &self.settings.excludes, self.settings.cap_bytes);
        snapshot_formatted(&self.git, folder, &self.target, excludes, cap, self.settings.formatter.as_ref())
            .map_err(snapshot_failure)?;
        save(&self.git, folder, &self.target, &join_save_subject(&self.target.request), excludes, cap)
            .map_err(snapshot_failure)
    }
}

/// SnapshotError has no Display, so its Debug form names the error.
fn snapshot_failure(error: SnapshotError) -> Report {
    failure_report(VERB, &format!("{error:?}"))
}
