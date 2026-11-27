//! The report of an `mr backup` or `mr restore-backup` that land refused, in the cli's own words: land's,
//! gitio's and oplog's Display texts are never printed; only a host, a refname, the remote, an op id or key, and
//! git's summary of a push refusal it did not classify, are.

use gitio::PushRefusal;
use land::BackupError;

use crate::landing::landing_refusal;
use crate::report::{failure_report, Report};

const RESTORE: &str = "restore-backup";

/// `verb` is "backup" or "restore-backup"; `remote` is the remote the verb talked to.
/// - Host, Git, Oplog: failure_report (exit 5) naming what failed; Git names the remote.
/// - BadHost: refused; on restore-backup the host named is the owner and the next step names it again; on backup
///   this machine's host name is the owner and the next step renames the machine.
/// - Refused: refused, why the remote refused to move the backup ref; a stale lease is repeated, the other refusals
///   are cleared on the remote first.
/// - OpFailed, OpRepeated: refused, naming the op id or the op key; the next step repeats the command.
///
/// Every refusal is exit 3, outcome "refused", with no fields.
pub fn backup_error_report(verb: &str, error: &BackupError, remote: &str) -> Report {
    match error {
        BackupError::Host(_) => failure_report(verb, "this machine's host name could not be read"),
        BackupError::BadHost(host) => {
            let (owner, next) = if verb == RESTORE {
                (
                    "the host named",
                    "name the host as its backup is named on the remote (refs/mortar-backup/<host>/): mr \
                     restore-backup <host> --actor <name>",
                )
            } else {
                (
                    "this machine's host name",
                    "give this machine a host name that holds no '/', then repeat the same command",
                )
            };
            let line = format!("host {host:?} cannot name a backup namespace: it is empty or holds '/'");
            landing_refusal(verb, vec![line], owner, next.to_string())
        }
        BackupError::Git(_) => {
            failure_report(verb, &format!("git failed while reading the refs or talking to the remote {remote}"))
        }
        BackupError::Refused { refname, refusal } => landing_refusal(
            verb,
            vec![format!("the remote {remote} refused to move {refname}: {}", refusal_words(refusal))],
            &format!("the remote {remote}"),
            refusal_next(refusal, remote),
        ),
        BackupError::Oplog(_) => failure_report(verb, "the op log could not be read or written"),
        BackupError::OpFailed(record) => landing_refusal(
            verb,
            vec![format!("op {} moved nothing: a local ref was not at its expected value", record.op_id)],
            "the local refs",
            "repeat the same command; it reads the refs afresh".to_string(),
        ),
        BackupError::OpRepeated(record) => landing_refusal(
            verb,
            vec![format!("op key {} is already done in the op log, so nothing moved", record.key)],
            "the op log",
            "repeat the same command".to_string(),
        ),
    }
}

/// Why the remote refused, in the cli's words; an unclassified refusal quotes git's summary.
fn refusal_words(refusal: &PushRefusal) -> String {
    match refusal {
        PushRefusal::StaleLease => "it moved on the remote since it was listed".to_string(),
        PushRefusal::NonFastForward => "the move is not a fast-forward".to_string(),
        PushRefusal::Hook => "a hook on the remote declined it".to_string(),
        PushRefusal::Other(summary) => format!("git's summary was {summary}"),
    }
}

/// A stale lease is listed afresh by a repeat; any other refusal stands until it is cleared on the remote.
fn refusal_next(refusal: &PushRefusal, remote: &str) -> String {
    match refusal {
        PushRefusal::StaleLease => "repeat the same command; it lists the backup afresh".to_string(),
        PushRefusal::NonFastForward | PushRefusal::Hook | PushRefusal::Other(_) => {
            format!("clear what the remote {remote} refuses for refs/mortar-backup/, then repeat the same command")
        }
    }
}
