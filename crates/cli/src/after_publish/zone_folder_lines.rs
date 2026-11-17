//! zone_folder_lines: what became of one named zone's folder, in the cli's words (S-139, S-140).

use std::path::PathBuf;

use gitio::Oid;

use super::{FolderRefusal, ZoneFolderOutcome};

/// One line for NoFolder, AtTip, Moved and Failed (with its why). A refused folder: why it was not brought to `tip`,
/// each path git or its index named (none for NoZoneCommit), then that nothing in it changed.
pub fn zone_folder_lines(zone: &str, tip: &Oid, outcome: &ZoneFolderOutcome) -> Vec<String> {
    match outcome {
        ZoneFolderOutcome::NoFolder => vec![format!("zone {zone} has no zone folder")],
        ZoneFolderOutcome::AtTip { folder } => {
            vec![format!("the zone folder {} is already at {tip}", folder.display())]
        }
        ZoneFolderOutcome::Moved { folder } => {
            vec![format!("brought the zone folder {} to {tip}", folder.display())]
        }
        ZoneFolderOutcome::Failed { folder, why } => {
            vec![format!("the zone folder {} was not brought to {tip}: {why}", folder.display())]
        }
        ZoneFolderOutcome::Refused { folder, why } => {
            let not_brought = format!("the zone folder {} was not brought to {tip}", folder.display());
            let (reason, paths): (String, &[PathBuf]) = match why {
                FolderRefusal::InTheWay(paths) => ("local changes are in the way at:".to_string(), paths),
                FolderRefusal::Unmerged(paths) => ("its index has unmerged entries at:".to_string(), paths),
                FolderRefusal::NoZoneCommit => (format!("its index matches no commit zone {zone} has held"), &[]),
            };
            std::iter::once(format!("{not_brought}: {reason}"))
                .chain(paths.iter().map(|path| path.display().to_string()))
                .chain(std::iter::once("nothing in it changed".to_string()))
                .collect()
        }
    }
}
