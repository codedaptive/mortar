//! The refusal of `mr commit-files` in a zone folder where a stopped run left its paths staged and zone/<zone> has
//! moved since. stopped_run_refusal_tests covers it.

use std::path::{Path, PathBuf};

use gitio::Oid;

use crate::report::{ExitCode, OutValue, Report};

/// Exit 1 "refused", owner "zone <zone>", fields zone and tip: a line naming the commit the paths were staged on
/// and the tip the branch has moved to, then that nothing was written. Next puts the paths back in the index as
/// `staged_on` has them, then refreshes the folder, then runs mr commit-files again. The refresh brings the folder
/// onto the tip unless the tip's move changed a path the folder has an edit in (a named path, whose stopped-run
/// content is then such an edit, or an unnamed edit): git refuses it and nothing is lost. So next says to move that
/// file aside and put it back after.
pub fn stopped_run_refusal(zone: &str, folder: &Path, staged_on: &Oid, tip: &Oid, paths: &[PathBuf]) -> Report {
    let shown: Vec<String> = paths.iter().map(|path| path.display().to_string()).collect();
    Report {
        verb: "commit-files".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![
            format!(
                "a stopped mr commit-files left the named paths staged on {staged_on}; zone/{zone} has moved to {tip} \
                 since"
            ),
            "nothing was written and no ref moved".to_string(),
        ],
        owner: Some(format!("zone {zone}")),
        next: format!(
            "git restore --staged --source={staged_on} -- {}, then mr folder refresh {} --actor <name> (if the \
             refresh refuses on a path, move that file aside and put it back after), then mr commit-files again",
            shown.join(" "),
            folder.display()
        ),
        fields: vec![
            ("zone".to_string(), OutValue::Text(zone.to_string())),
            ("tip".to_string(), OutValue::Text(tip.to_string())),
        ],
    }
}
