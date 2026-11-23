//! The report of a join refused because the new merge of the zone with its partners conflicted.

use std::path::PathBuf;

use crate::report::{with_moved_first, ExitCode, OutValue, Report};

/// Exit 1 "refused", owned by the coordinators of both zones, since the conflict is between their tips and no
/// change in the request can settle it: the conflicted files, indented, after a line naming `zone` and `partner`,
/// then that join moved nothing. `moved` (moved_before's line), when given, is the first line: it names the refs the
/// save or snapshot before the join moved. Fields: partner, files.
pub(crate) fn partners_conflicted_report(zone: &str, partner: &str, files: &[PathBuf], moved: Option<&str>) -> Report {
    let paths: Vec<String> = files.iter().map(|file| file.display().to_string()).collect();
    let mut lines = vec![format!("the new merge of zone {zone} with partner {partner} conflicts on these files:")];
    lines.extend(paths.iter().map(|path| format!("  {path}")));
    lines.push("join moved no ref and wrote no state".to_string());
    Report {
        verb: "join".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: with_moved_first(moved, lines),
        owner: Some(format!("the coordinators of zones {zone} and {partner}")),
        next: format!("repeat mr join once zones {zone} and {partner} agree on these files"),
        fields: vec![
            ("partner".to_string(), OutValue::Text(partner.to_string())),
            ("files".to_string(), OutValue::Texts(paths)),
        ],
    }
}
