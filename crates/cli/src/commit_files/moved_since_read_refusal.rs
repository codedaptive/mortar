//! The refusal of `mr commit-files` in a zone folder whose op found zone/<zone> moved after the run read it.
//! moved_since_read_refusal_tests covers it.

use std::path::{Path, PathBuf};

use gitio::Oid;

use crate::report::{ExitCode, Report};

/// Exit 3 "refused", owner "zone <zone>", no fields: zone/<zone> moved after the run read it at `tip`, so no ref
/// moved, and the named paths were left staged as committed onto `tip`. Next puts them back as `tip` has them, then
/// has mr folder refresh bring the folder to the new tip, then runs mr commit-files again. The refresh refuses, with
/// nothing lost, when the new tip changed a path the folder has an edit in, named or not (git refuses an edit in the
/// way); next says to move that file aside for the refresh and put it back after.
pub fn moved_since_read_refusal(zone: &str, folder: &Path, tip: &Oid, paths: &[PathBuf]) -> Report {
    let shown: Vec<String> = paths.iter().map(|path| path.display().to_string()).collect();
    Report {
        verb: "commit-files".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![
            format!("zone/{zone} moved since it was read at {tip}, so no ref moved"),
            format!("the named paths were left staged as committed onto {tip}"),
        ],
        owner: Some(format!("zone {zone}")),
        next: format!(
            "git restore --staged --source={tip} -- {}, then mr folder refresh {} --actor <name> (if the refresh \
             refuses on a path, move that file aside and put it back after), then mr commit-files again",
            shown.join(" "),
            folder.display()
        ),
        fields: Vec::new(),
    }
}
