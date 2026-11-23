//! The report of an `mr folder refresh` that set the zone folder's paths and brought it to its zone branch's tip
//! (zone_folder_move's bring_zone_folder).

use std::path::Path;

use gitio::Oid;

use crate::report::{ExitCode, OutValue, Report};
use crate::zone_folder_move::{folder_part, zone_folder_lines, ZoneFolderOutcome, FOLDER_PARTS};

/// Lines: the paths set ("set the zone folder <folder>'s paths to: ...") when `paths_changed`, else that the folder
/// already holds the zone's paths; then zone_folder_lines at `tip`. Failed is exit Failure "folder-failed", Refused
/// exit Refused "folder-not-moved", AtTip with no paths changed Done "already-refreshed", Moved or AtTip after a paths
/// change Done "refreshed". NoFolder, which bring_zone_folder never answers, is "folder-failed" too. The owner is the
/// folder unless Done. next: the folder part the outcome needs (FOLDER_PARTS), then the repeat of this command; Done
/// is "nothing to do". Fields: folder, zone, tip, paths, paths_changed, and moved (true only for Moved).
pub fn refresh_report(
    zone: &str,
    folder: &Path,
    tip: &Oid,
    paths: &[String],
    paths_changed: bool,
    outcome: &ZoneFolderOutcome,
) -> Report {
    let shown = folder.display();
    let first = match paths_changed {
        true => format!("set the zone folder {shown}'s paths to: {}", paths.join(", ")),
        false => format!("the zone folder {shown} already holds zone {zone}'s paths"),
    };
    let (exit, word) = match outcome {
        ZoneFolderOutcome::Failed { .. } | ZoneFolderOutcome::NoFolder => (ExitCode::Failure, "folder-failed"),
        ZoneFolderOutcome::Refused { .. } => (ExitCode::Refused, "folder-not-moved"),
        ZoneFolderOutcome::AtTip { .. } if !paths_changed => (ExitCode::Done, "already-refreshed"),
        ZoneFolderOutcome::AtTip { .. } | ZoneFolderOutcome::Moved { .. } => (ExitCode::Done, "refreshed"),
    };
    let repeat = format!("repeat the same command: mr folder refresh {shown} --actor <name>");
    let next = match (folder_part(outcome), exit) {
        (Some(part), _) => format!("{}; then {repeat}", FOLDER_PARTS[part]),
        (None, ExitCode::Done) => "nothing to do".to_string(),
        (None, _) => repeat,
    };
    Report {
        verb: "folder".to_string(),
        exit,
        outcome: word.to_string(),
        lines: std::iter::once(first).chain(zone_folder_lines(zone, tip, outcome)).collect(),
        owner: (exit != ExitCode::Done).then(|| format!("the zone folder {shown}")),
        next,
        fields: vec![
            ("folder".to_string(), OutValue::Text(shown.to_string())),
            ("zone".to_string(), OutValue::Text(zone.to_string())),
            ("tip".to_string(), OutValue::Text(tip.to_string())),
            ("paths".to_string(), OutValue::Texts(paths.to_vec())),
            ("paths_changed".to_string(), OutValue::Bool(paths_changed)),
            ("moved".to_string(), OutValue::Bool(matches!(outcome, ZoneFolderOutcome::Moved { .. }))),
        ],
    }
}
