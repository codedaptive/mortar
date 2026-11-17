//! The report of an `mr restore-backup` that land's restore_backup answered (SPEC-cli 06).

use land::Restored;

use crate::report::{ExitCode, OutValue, Report};

/// Exit 0 "restored", no owner. No entries: one line saying `host` has no backup on the remote. Else one line per
/// entry, in land's order. The next step names the host's backup namespace when any ref differs from its backup
/// (it was left as it is); else "nothing to do". Fields created, same, differs and vanished: the refnames of each
/// kind, in order.
pub fn restore_backup_report(host: &str, restored: &[Restored]) -> Report {
    let mut lines = Vec::new();
    let (mut created, mut same, mut differs, mut vanished) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    for entry in restored {
        match entry {
            Restored::Created { refname, oid } => {
                lines.push(format!("created {refname} at {oid}"));
                created.push(refname.clone());
            }
            Restored::Same { refname } => {
                lines.push(format!("{refname} already holds the backup's commit"));
                same.push(refname.clone());
            }
            Restored::Differs { refname, local, backup } => {
                lines.push(format!("{refname} is at {local}, its backup at {backup}: left as it is"));
                differs.push(refname.clone());
            }
            Restored::Vanished { refname } => {
                lines.push(format!("{refname} was gone from the remote by the fetch: not created"));
                vanished.push(refname.clone());
            }
        }
    }
    if lines.is_empty() {
        lines.push(format!("host {host} has no backup on the remote"));
    }
    let next = if differs.is_empty() {
        "nothing to do".to_string()
    } else {
        format!(
            "each ref left as it is holds another commit than its backup under refs/mortar-backup/{host}/ on the \
             remote; keep it, or move it to the backup's commit by hand"
        )
    };
    Report {
        verb: "restore-backup".to_string(),
        exit: ExitCode::Done,
        outcome: "restored".to_string(),
        lines,
        owner: None,
        next,
        fields: vec![
            ("created".to_string(), OutValue::Texts(created)),
            ("same".to_string(), OutValue::Texts(same)),
            ("differs".to_string(), OutValue::Texts(differs)),
            ("vanished".to_string(), OutValue::Texts(vanished)),
        ],
    }
}
