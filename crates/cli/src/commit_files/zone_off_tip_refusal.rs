//! The refusal of `mr commit-files` in a zone folder that cannot be shown to stand on its zone branch's tip.
//! zone_off_tip_refusal_tests covers it.

use std::path::Path;

use gitio::Oid;

use crate::report::{ExitCode, OutValue, Report};
use crate::zone_folder_move::ZoneOld;

/// `old` is find_zone_old's answer for the zone folder's index tree. AtTip gives None. From and NoMatch are exit 3
/// "refused", owner "zone <zone>", fields zone and tip: a line saying where the folder stands, then that nothing was
/// written.
/// - From(v): the folder stands on v, a commit the branch held before its tip; next refreshes it onto the tip.
/// - NoMatch: the zone folder's HEAD follows zone/<zone>, so only its index tells where the folder stands, and an
///   index at no commit the branch has held does not tell it. A staged named path can hide a lag: a folder on an
///   older commit with a named path staged can match the tip with that path put on it, and committing onto the tip
///   would then overwrite the tip's own change to that path. It refuses with no refresh as the next step, since
///   bring_zone_folder refuses the same state (NoZoneCommit); a run of mr commit-files stopped after staging its
///   paths is told apart before this by the runner's pending marker. Next has the caller see with git diff what the
///   commit would change on the tip, and unstage the paths when that is only their own change.
pub fn zone_off_tip_refusal(zone: &str, folder: &Path, tip: &Oid, old: &ZoneOld) -> Option<Report> {
    let folder = folder.display();
    let (line, next) = match old {
        ZoneOld::AtTip => return None,
        ZoneOld::From(stands_on) => (
            format!("the zone folder {folder} stands on {stands_on}, not on zone/{zone}'s tip {tip}"),
            format!("mr folder refresh {folder} --actor <name>, then mr commit-files again"),
        ),
        ZoneOld::NoMatch => (
            format!(
                "the zone folder {folder}'s index holds staged changes at no commit zone/{zone} has held, so the \
                 commit it stands on cannot be told"
            ),
            format!(
                "git diff zone/{zone} -- <path>... shows what the commit would change on the tip; if that is only \
                 your change, git restore --staged -- <path>... and run mr commit-files again"
            ),
        ),
    };
    Some(Report {
        verb: "commit-files".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![line, "nothing was written and no ref moved".to_string()],
        owner: Some(format!("zone {zone}")),
        next,
        fields: vec![
            ("zone".to_string(), OutValue::Text(zone.to_string())),
            ("tip".to_string(), OutValue::Text(tip.to_string())),
        ],
    })
}
