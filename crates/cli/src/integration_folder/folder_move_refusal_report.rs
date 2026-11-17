//! The report of an integration folder that could not move after its branch moved.

use std::path::Path;

use gitio::Oid;

use crate::report::{ExitCode, OutValue, Report};

/// Exit 1 "folder-not-moved": `done` (what the verb already did), then that the integration folder at `folder`
/// could not move from `old` to `new` and `why`. The branch already stands at `new` and no Mortar verb moves a
/// folder that has no folder record, so the next is the owner's step in words: clear what stops the folder, then
/// bring its files and index to `new`. Fields: the folder, old and new as text.
pub fn folder_move_refusal_report(verb: &str, done: &str, folder: &Path, old: &Oid, new: &Oid, why: &str) -> Report {
    let shown = folder.display();
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Refused,
        outcome: "folder-not-moved".to_string(),
        lines: vec![
            done.to_string(),
            format!("the integration folder {shown} could not move from {old} to {new}: {why}"),
        ],
        owner: Some(format!("the integration folder {shown}")),
        next: format!(
            "clear what stops the integration folder {shown} from moving, then bring its files and index to {new}, \
             the commit integration now points at; integration itself has already moved"
        ),
        fields: vec![
            ("folder".to_string(), OutValue::Text(shown.to_string())),
            ("old".to_string(), OutValue::Text(old.to_string())),
            ("new".to_string(), OutValue::Text(new.to_string())),
        ],
    }
}
