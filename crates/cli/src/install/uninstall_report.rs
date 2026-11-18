//! uninstall_report: Uninstall's report (spec 07, "mr uninstall").

use std::path::Path;

use crate::report::{ExitCode, OutValue, Report};

use super::{item_json, item_line, InstallItem};

/// Exit 0: "not-installed" when there was no record, else "uninstalled" with a line per item. `--json` carries the
/// items as `items`, as install's report does. `next` names `mortar install`.
pub fn uninstall_report(record: &Path, items: Option<&[InstallItem]>) -> Report {
    let (outcome, lines, next, items) = match items {
        None => (
            "not-installed",
            vec![format!("nothing to remove: no install record at {}", record.display())],
            "mortar install",
            &[][..],
        ),
        Some(items) => ("uninstalled", items.iter().map(item_line).collect(), "mortar install, to put Mortar back", items),
    };
    Report {
        verb: "uninstall".to_string(),
        exit: ExitCode::Done,
        outcome: outcome.to_string(),
        lines,
        owner: None,
        next: next.to_string(),
        fields: vec![("items".to_string(), OutValue::Raw(items.iter().map(item_json).collect()))],
    }
}
