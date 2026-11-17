//! The report of `mr disk`: each folder's bytes, the store once, the refs by namespace and what a plain sweep
//! frees, in the cli's words. folders' SweepEstimate reason is never printed: it is folders' text, not the cli's.

use folders::DiskReport;

use crate::folder::folder_kind_text;
use crate::report::{ExitCode, OutValue, Report};

/// Exit 0, outcome "measured". One line per folder (its path, kind, zone and request, files and build bytes), then
/// "object store: <n> bytes, counted once", one line of ref counts by namespace, and the sweep figure as what a
/// mr sweep with no --idle-days frees. The next is a sweep of idle build dirs when any folder has build bytes,
/// else "nothing to do". Fields: folders (paths), store_bytes, refs ("<namespace> <count>"), sweep_bytes.
pub fn disk_report(report: &DiskReport) -> Report {
    let mut lines: Vec<String> = report
        .folders
        .iter()
        .map(|folder| {
            format!(
                "{}: {}; files {} bytes, build {} bytes",
                folder.path.display(),
                folder_kind_text(&folder.record),
                folder.files_bytes,
                folder.build_bytes
            )
        })
        .collect();
    let refs: Vec<String> = report.refs.iter().map(|(namespace, count)| format!("{namespace} {count}")).collect();
    lines.push(format!("object store: {} bytes, counted once", report.store_bytes));
    lines.push(format!("refs by namespace: {}", refs.join(", ")));
    lines.push(format!("an mr sweep with no --idle-days frees up to {} bytes", report.sweep.bytes));
    let next = if report.folders.iter().any(|folder| folder.build_bytes > 0) {
        "mr sweep --idle-days <n> --actor <name>"
    } else {
        "nothing to do"
    };
    Report {
        verb: "disk".to_string(),
        exit: ExitCode::Done,
        outcome: "measured".to_string(),
        lines,
        owner: None,
        next: next.to_string(),
        fields: vec![
            (
                "folders".to_string(),
                OutValue::Texts(report.folders.iter().map(|folder| folder.path.display().to_string()).collect()),
            ),
            ("store_bytes".to_string(), OutValue::Number(report.store_bytes)),
            ("refs".to_string(), OutValue::Texts(refs)),
            ("sweep_bytes".to_string(), OutValue::Number(report.sweep.bytes)),
        ],
    }
}
