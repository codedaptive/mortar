//! The report of an integration folder that could not move: after a verb moved its branch, or at the finish of a
//! move a stopped run left undone, before the verb's land call.

use std::path::Path;

use gitio::Oid;

use super::MoveBlock;
use crate::report::{ExitCode, OutValue, Report};

/// Exit 1 "folder-not-moved": `done` (what the verb did, given by the caller), then that the integration folder at
/// `folder` could not move from `old` to `new`, what blocked it, and that `verb` stopped. For
/// MoveBlock::Paths the paths in the way follow one per line, and the next is to move each out or put it back to
/// the folder's index copy, leaving the index as it is; for MoveBlock::Other it is to clear what stops the folder,
/// leaving its index as it is. Either way the next ends with `repeat`, the same command run again, which finishes
/// the move when the folder was cleared as the next says (a clear that resets the index to HEAD leaves no stall for
/// the repeat to see, and the files are then their owner's, unreported: S-137 (3), (4)); it names no git command
/// (S-112). Fields: the folder, old and new as text, and for Paths the paths as texts.
pub fn folder_move_refusal_report(
    verb: &str,
    done: &str,
    folder: &Path,
    old: &Oid,
    new: &Oid,
    block: &MoveBlock,
    repeat: &str,
) -> Report {
    let shown = folder.display();
    let mut fields = vec![
        ("folder".to_string(), OutValue::Text(shown.to_string())),
        ("old".to_string(), OutValue::Text(old.to_string())),
        ("new".to_string(), OutValue::Text(new.to_string())),
    ];
    let mut lines = vec![done.to_string()];
    let next = match block {
        MoveBlock::Paths(paths) => {
            let texts: Vec<String> = paths.iter().map(|path| path.display().to_string()).collect();
            lines.push(format!(
                "the integration folder {shown} could not move from {old} to {new}: local changes are in the way at:"
            ));
            lines.extend(texts.iter().cloned());
            fields.push(("paths".to_string(), OutValue::Texts(texts)));
            format!(
                "move each path listed out of the integration folder {shown}, or put it back to the folder's index \
                 copy, and leave the index as it is; then repeat the same command: {repeat}"
            )
        }
        MoveBlock::Other(why) => {
            lines.push(format!("the integration folder {shown} could not move from {old} to {new}: {why}"));
            format!(
                "clear what stops the integration folder {shown} from moving, leaving its index as it is; then repeat \
                 the same command: {repeat}"
            )
        }
    };
    lines.push(format!("{verb} stopped here"));
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Refused,
        outcome: "folder-not-moved".to_string(),
        lines,
        owner: Some(format!("the integration folder {shown}")),
        next,
        fields,
    }
}
