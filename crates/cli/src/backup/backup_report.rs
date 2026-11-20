//! The report of an `mr backup` that land's backup answered.

use land::Backup;

use crate::report::{ExitCode, OutValue, Report};

/// Exit 0 "backed-up", next "nothing to do". Nothing pushed: one line saying the host's backup already holds every
/// in-flight ref; else a count line, then each pushed refname on its own line. Always last: how many refs were
/// already backed up. Fields pushed and unchanged, the refnames as land gave them.
pub fn backup_report(answer: &Backup) -> Report {
    let mut lines = if answer.pushed.is_empty() {
        vec!["nothing to update: this host's backup on the remote already holds every in-flight ref".to_string()]
    } else {
        let mut lines = vec![format!("updated {} refs in this host's backup on the remote:", answer.pushed.len())];
        lines.extend(answer.pushed.iter().cloned());
        lines
    };
    lines.push(format!("{} refs were already backed up", answer.unchanged.len()));
    Report {
        verb: "backup".to_string(),
        exit: ExitCode::Done,
        outcome: "backed-up".to_string(),
        lines,
        owner: None,
        next: "nothing to do".to_string(),
        fields: vec![
            ("pushed".to_string(), OutValue::Texts(answer.pushed.clone())),
            ("unchanged".to_string(), OutValue::Texts(answer.unchanged.clone())),
        ],
    }
}
