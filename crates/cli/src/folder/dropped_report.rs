//! The report of an `mr folder drop` that dropped every attempt, or found none left.

use std::path::PathBuf;

use crate::report::{ExitCode, OutValue, Report};

/// Exit 0; outcome "dropped", or "already-done" when `dropped` is empty. Lines: each re-made orphan attempt folder
/// in `remade`; the attempts dropped (their folders and try branches gone, their trysnap refs kept), or that no
/// attempt was left. Fields: dropped (the attempt numbers), remade. Next: go on with the request in its own folder.
pub fn dropped_report(dropped: &[u32], remade: &[PathBuf]) -> Report {
    let mut lines: Vec<String> =
        remade.iter().map(|path| format!("re-made the orphan attempt folder {}", path.display())).collect();
    let numbers: Vec<String> = dropped.iter().map(u32::to_string).collect();
    lines.push(if dropped.is_empty() {
        "no attempt was left to drop".to_string()
    } else {
        let numbers = numbers.join(", ");
        format!("dropped attempts {numbers}: their folders and try branches are gone, their trysnap refs kept")
    });
    Report {
        verb: "folder".to_string(),
        exit: ExitCode::Done,
        outcome: if dropped.is_empty() { "already-done" } else { "dropped" }.to_string(),
        lines,
        owner: None,
        next: "go on with the request in its own folder".to_string(),
        fields: vec![
            ("dropped".to_string(), OutValue::Raw(numbers)),
            ("remade".to_string(), OutValue::Texts(remade.iter().map(|path| path.display().to_string()).collect())),
        ],
    }
}
