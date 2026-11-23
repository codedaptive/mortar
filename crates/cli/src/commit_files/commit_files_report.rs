//! The report of an `mr commit-files` that committed the named paths, or found them already held.
//! commit_files_report_tests covers it.

use std::path::PathBuf;

use gitio::Oid;

use crate::report::{ExitCode, OutValue, Report};

/// Exit 0, no owner, `next` as given. Fields commit, ref, moved and paths, in that order.
/// - Some(c): outcome "committed"; the commit, the ref's move from `old` to c, and that the folder's other changes
///   were left as they are; commit is c and moved true.
/// - None: outcome "same"; the ref at `old` already holds the paths, nothing written, no ref moved; commit is `old`
///   and moved false.
pub fn commit_files_report(refname: &str, old: &Oid, commit: Option<&Oid>, paths: &[PathBuf], next: String) -> Report {
    let (outcome, lines, shown, moved) = match commit {
        Some(commit) => {
            let noun = if paths.len() == 1 { "path" } else { "paths" };
            let lines = vec![
                format!(
                    "committed {} named {noun} as {commit}: {refname} moved from {old} to {commit}",
                    paths.len()
                ),
                "the folder's other changes were left as they are".to_string(),
            ];
            ("committed", lines, commit, true)
        }
        None => {
            let line = format!(
                "{refname} at {old} already holds the named paths as the folder has them; nothing was written and \
                 no ref moved"
            );
            ("same", vec![line], old, false)
        }
    };
    Report {
        verb: "commit-files".to_string(),
        exit: ExitCode::Done,
        outcome: outcome.to_string(),
        lines,
        owner: None,
        next,
        fields: vec![
            ("commit".to_string(), OutValue::Text(shown.to_string())),
            ("ref".to_string(), OutValue::Text(refname.to_string())),
            ("moved".to_string(), OutValue::Bool(moved)),
            ("paths".to_string(), OutValue::Texts(paths.iter().map(|path| path.display().to_string()).collect())),
        ],
    }
}
