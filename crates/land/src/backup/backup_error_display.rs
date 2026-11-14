//! Display for BackupError: each variant says why and names what it carries; the refusals backup and restore
//! make themselves also say what moved. backup_error_display_tests cover BadHost, Refused,
//! OpFailed and OpRepeated; the Host, Git and Oplog texts are Nagatha's RITs (owed, S-101).

use crate::backup::BackupError;
use gitio::PushRefusal;
use std::fmt;

impl fmt::Display for BackupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackupError::Host(e) => write!(f, "backup host name: {e}; nothing was pushed or restored"),
            BackupError::BadHost(host) => write!(
                f,
                "host name {host:?} is empty or holds '/', so it cannot name a backup namespace; nothing was \
                 pushed or restored"
            ),
            BackupError::Git(e) => write!(f, "backup git: {e}"),
            BackupError::Refused { refname, refusal } => write!(
                f,
                "the remote refused to move backup ref {refname}: {}; it was not moved, other backup refs in the \
                 same push may have been; run backup again to compare afresh",
                refusal_why(refusal)
            ),
            BackupError::Oplog(e) => write!(f, "backup op log: {e}"),
            BackupError::OpFailed(record) => write!(
                f,
                "op {} (key {}) failed: a local ref was not at its expected value, so nothing was restored; run \
                 restore again to read the refs afresh",
                record.op_id, record.key
            ),
            BackupError::OpRepeated(record) => write!(
                f,
                "op key {} is already done in the op log, so this restore moved nothing; run restore again",
                record.key
            ),
        }
    }
}

/// Our words for each refusal: PushRefusal has no Display of its own.
fn refusal_why(refusal: &PushRefusal) -> String {
    match refusal {
        PushRefusal::StaleLease => "it moved on the remote since it was read (stale lease)".to_string(),
        PushRefusal::NonFastForward => "the move is not a fast-forward".to_string(),
        PushRefusal::Hook => "a hook on the remote declined it".to_string(),
        PushRefusal::Other(summary) => format!("git said {summary}"),
    }
}
