//! Whether an `mr restore` must refuse before moving a ref, for the folders standing on the refs it moves.

use std::path::PathBuf;

use folders::FolderKind;
use oplog::RefMove;

use super::FolderOnRef;
use crate::report::{is_zero_oid, ExitCode, Report};

/// Refused (exit 3, nothing moved) when a request or attempt folder stands on a ref the restore would delete (a
/// move with an all-zero new), or a folder in `unsaved` (the caller puts there each folder needing a move whose work
/// is not saved, or holds a file the size cap refused). One line per folder and why, in `folders`' order, then
/// "nothing was moved". Zone folders are neither checked nor moved. The next step is in words:
/// saving comes first when any folder is unsaved. None when nothing stands in the way.
pub fn restore_refusal(folders: &[FolderOnRef], unsaved: &[PathBuf], moves: &[RefMove]) -> Option<Report> {
    let deletes = |refname: &str| moves.iter().any(|m| m.refname == refname && is_zero_oid(&m.new));
    let (mut lines, mut any_unsaved) = (Vec::new(), false);
    for folder in folders.iter().filter(|folder| folder.record.kind != FolderKind::Zone) {
        let (path, refname) = (folder.path.display(), &folder.refname);
        if unsaved.contains(&folder.path) {
            any_unsaved = true;
            lines.push(format!("the folder {path} on {refname} holds work that is not saved"));
        }
        if deletes(refname) {
            lines.push(format!("the folder {path} stands on {refname}, which the restore would delete"));
        }
    }
    if lines.is_empty() {
        return None;
    }
    lines.push("nothing was moved".to_string());
    let next = if any_unsaved {
        "have the folder's agent run mr save, then repeat"
    } else {
        "restore without the refs those folders stand on, or have the harness remove those folders first, then repeat"
    };
    Some(Report {
        verb: "restore".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines,
        owner: Some("the agent working in each folder named".to_string()),
        next: next.to_string(),
        fields: Vec::new(),
    })
}
