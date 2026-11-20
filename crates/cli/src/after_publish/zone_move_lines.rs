//! zone_move_lines: what land's after_publish did with one zone, in the cli's words.

use land::ZoneMove;

/// Moved: the zone's move from old to new, then one line per open request that rebases at its next join. Current:
/// the zone already holds origin and its branch did not move. Conflicted: the zone left in place, the commit whose
/// replay conflicts, then each conflicted file.
pub fn zone_move_lines(zone_move: &ZoneMove) -> Vec<String> {
    match zone_move {
        ZoneMove::Moved { zone, old, new, rebase } => std::iter::once(format!("zone {zone} moved from {old} to {new}"))
            .chain(rebase.iter().map(|request| format!("request {} rebases at its next join", request.name)))
            .collect(),
        ZoneMove::Current { zone, tip } => {
            vec![format!("zone {zone} already holds origin at {tip}; its branch did not move")]
        }
        ZoneMove::Conflicted { zone, tip, commit, files } => {
            std::iter::once(format!("zone {zone} left in place at {tip}: replaying {commit} onto origin conflicts in:"))
                .chain(files.iter().map(|file| file.display().to_string()))
                .collect()
        }
    }
}
