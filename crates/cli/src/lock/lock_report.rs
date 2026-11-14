//! The report of a verb whose lock could not be taken (S-71: busy is exit 3).
//! Shell API with no caller yet but locked_act: the ref-moving verbs of WO-cli-2..6 reach it through that.

use oplog::{LockError, LockRecord};

use crate::report::{failure_report, ExitCode, OutValue, Report};

const LATER: &str = "repeat the same command later";

/// Held and TakeoverInProgress: busy, naming the file and its holder's pid, host, op key and since (unix
/// seconds), owner "pid <pid> on <host>", with the holder as --json fields. Contended: busy, no holder. BadName:
/// refused, the zone cannot name a lock file. Every other lock error: failure.
pub fn lock_report(verb: &str, error: &LockError) -> Report {
    match error {
        LockError::Held { path, holder, .. } => {
            busy_by(verb, format!("the lock {} is held by {}", path.display(), holder_text(holder)), holder)
        }
        LockError::TakeoverInProgress { path, holder, .. } => busy_by(
            verb,
            format!("a takeover of the lock is in progress: {} is held by {}", path.display(), holder_text(holder)),
            holder,
        ),
        LockError::Contended { path, .. } => Report {
            verb: verb.to_string(),
            exit: ExitCode::Busy,
            outcome: "busy".to_string(),
            lines: vec![format!("the lock {} kept changing hands; nothing ran", path.display())],
            owner: None,
            next: LATER.to_string(),
            fields: Vec::new(),
        },
        LockError::BadName(zone) => Report {
            verb: verb.to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![format!("the zone {zone:?} cannot name a lock file; nothing ran")],
            owner: Some("the zones in the Mortar config".to_string()),
            next: "name zones with letters, digits, '-', '_' and '.', not starting with '.'".to_string(),
            fields: Vec::new(),
        },
        other => failure_report(verb, &other.to_string()),
    }
}

fn holder_text(holder: &LockRecord) -> String {
    format!(
        "pid {} on {} for op {} since {} (unix seconds); nothing ran",
        holder.pid, holder.host, holder.key, holder.since
    )
}

fn busy_by(verb: &str, line: String, holder: &LockRecord) -> Report {
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Busy,
        outcome: "busy".to_string(),
        lines: vec![line],
        owner: Some(format!("pid {} on {}", holder.pid, holder.host)),
        next: LATER.to_string(),
        fields: vec![
            ("holder_pid".to_string(), OutValue::Number(u64::from(holder.pid))),
            ("holder_host".to_string(), OutValue::Text(holder.host.clone())),
            ("holder_key".to_string(), OutValue::Text(holder.key.clone())),
            ("since".to_string(), OutValue::Number(holder.since)),
        ],
    }
}
