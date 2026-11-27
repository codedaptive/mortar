//! The refusal of an `mr folder request --with` whose merge of the zone's tip with a partner zone's tip conflicts.

use std::path::PathBuf;

use crate::report::{ExitCode, OutValue, Report};

/// Exit 3 "refused": the merge of `zone` with `partner` conflicts on `files` (one line each, indented two spaces)
/// and nothing was made. The coordinators of both zones own it; the next step cuts the request without the partner
/// or repeats once the zones agree. Fields: partner, then files.
pub fn partner_conflict_report(zone: &str, partner: &str, files: &[PathBuf]) -> Report {
    let files: Vec<String> = files.iter().map(|file| file.display().to_string()).collect();
    let mut lines = vec![format!("the merge of zone {zone} with partner {partner} conflicts on these files:")];
    lines.extend(files.iter().map(|file| format!("  {file}")));
    lines.push("nothing was made".to_string());
    Report {
        verb: "folder".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines,
        owner: Some(format!("the coordinators of zones {zone} and {partner}")),
        next: format!(
            "cut the request without --with {partner}, or repeat once zones {zone} and {partner} agree on these files"
        ),
        fields: vec![
            ("partner".to_string(), OutValue::Text(partner.to_string())),
            ("files".to_string(), OutValue::Texts(files)),
        ],
    }
}
