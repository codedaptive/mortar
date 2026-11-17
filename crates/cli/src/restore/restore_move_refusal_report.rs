//! The report of a folder move that failed after `mr restore` moved the refs (S-113(3)).

use std::path::{Path, PathBuf};

use crate::report::{ExitCode, Report};

/// Exit 1 "refused": the refs are restored (op `op_id`); each folder already `moved`; then the folder at `folder` on
/// `refname` that could not move, and `why` (the move's error, as folders words it). A plain repeat of the same
/// command finishes the move: oplog answers Repeated with the restore's moves, and the folder is moved from them.
pub fn restore_move_refusal_report(op_id: &str, moved: &[PathBuf], folder: &Path, refname: &str, why: &str) -> Report {
    let mut lines = vec![format!("the refs are restored as of op {op_id}")];
    lines.extend(moved.iter().map(|path| format!("moved the folder {} to its ref's restored value", path.display())));
    lines.push(format!("the folder {} on {refname} could not move: {why}", folder.display()));
    Report {
        verb: "restore".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines,
        owner: Some(format!("the agent working in {}", folder.display())),
        next: "clear what stops the folder's move, then repeat the same mr restore command; the repeat finishes \
               the move"
            .to_string(),
        fields: Vec::new(),
    }
}
