//! The report of a `mortar folder attempts` that made (or found already made) its attempt folders.

use std::path::{Path, PathBuf};

use folders::FolderRecord;

use crate::report::{ExitCode, OutValue, Report};

/// Exit 0, outcome "made". Lines: how many attempt folders are under `dir`, then one per attempt, its path and base
/// (full commit id). Fields: dir, attempts (the paths, in `records`' order). Next: start one agent in each attempt
/// folder, then keep one or drop them all from the request's folder; `<k>` and `<name>` are printed as written.
pub fn attempts_made_report(dir: &Path, records: &[(PathBuf, FolderRecord)]) -> Report {
    let mut lines = vec![format!("{} attempt folders are under {}", records.len(), dir.display())];
    lines.extend(records.iter().map(|(path, record)| {
        format!("attempt folder {} on base {}", path.display(), record.base.as_str())
    }));
    Report {
        verb: "folder".to_string(),
        exit: ExitCode::Done,
        outcome: "made".to_string(),
        lines,
        owner: None,
        next: "start one agent in each attempt folder; then mortar folder keep <k> --actor <name> or mortar folder \
               drop --actor <name> in the request's folder"
            .to_string(),
        fields: vec![
            ("dir".to_string(), OutValue::Text(dir.display().to_string())),
            (
                "attempts".to_string(),
                OutValue::Texts(records.iter().map(|(path, _)| path.display().to_string()).collect()),
            ),
        ],
    }
}
