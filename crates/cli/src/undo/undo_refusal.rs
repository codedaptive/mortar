//! undo_refusal: the report of an undo refused by what choose_undo found, before any snapshot or op log write
//! (S-73; S-142 item 1).

use super::UndoChoice;
use crate::report::{ExitCode, Report};

const NOTHING_CHANGED: &str = "nothing was changed";

/// Refused (exit 1), nothing changed, with a next an agent can follow: ZoneOwns, to the zone's coordinator, who can
/// back the join out; Resolved, the way forward in words, since undo cannot put a paused join back; NotAStep,
/// naming the op and its kind, next mortar status. None for Nothing, Step and AlreadyUndone.
pub fn undo_refusal(choice: &UndoChoice) -> Option<Report> {
    match choice {
        UndoChoice::Nothing | UndoChoice::Step { .. } | UndoChoice::AlreadyUndone { .. } => None,
        UndoChoice::ZoneOwns { op_id } => Some(refused(
            format!("op {op_id} is a completed join: the zone owns it now"),
            Some("the zone's coordinator"),
            format!("ask the zone's coordinator to back out op {op_id}"),
        )),
        UndoChoice::Resolved { op_id } => Some(refused(
            format!("op {op_id} resolved a join, and undo cannot put the paused join back"),
            None,
            "change the files, mortar save, then mortar join".to_string(),
        )),
        UndoChoice::NotAStep { op_id, kind } => Some(refused(
            format!("op {op_id} is a {kind}: only a save, a rebase or a resolve of this folder can be undone"),
            None,
            "mortar status".to_string(),
        )),
    }
}

fn refused(why: String, owner: Option<&str>, next: String) -> Report {
    Report {
        verb: "undo".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![why, NOTHING_CHANGED.to_string()],
        owner: owner.map(str::to_string),
        next,
        fields: Vec::new(),
    }
}
