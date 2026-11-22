//! refresh_report: each outcome of a refresh (paths set or already held, then the move's answer) with its exit,
//! outcome word, lines, owner, next and fields.

use std::path::PathBuf;

use gitio::Oid;

use super::refresh_report;
use crate::report::{ExitCode, OutValue, Report};
use crate::zone_folder_move::{FolderRefusal, ZoneFolderOutcome};

const TIP: &str = "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391";
const SET: &str = "set the zone folder /w/zone-core's paths to: src, sub, root.txt";
const HELD: &str = "the zone folder /w/zone-core already holds zone core's paths";
const NOT_BROUGHT: &str = "the zone folder /w/zone-core was not brought to e69de29bb2d1d6434b8b29ae775ad8c2e48c5391";

fn folder() -> PathBuf {
    PathBuf::from("/w/zone-core")
}

fn expected(
    exit: ExitCode,
    outcome: &str,
    lines: &[&str],
    next: &str,
    (paths_changed, moved): (bool, bool),
) -> Report {
    let text = |value: &str| OutValue::Text(value.to_string());
    Report {
        verb: "folder".to_string(),
        exit,
        outcome: outcome.to_string(),
        lines: lines.iter().map(ToString::to_string).collect(),
        owner: (exit != ExitCode::Done).then(|| "the zone folder /w/zone-core".to_string()),
        next: next.to_string(),
        fields: vec![
            ("folder".to_string(), text("/w/zone-core")),
            ("zone".to_string(), text("core")),
            ("tip".to_string(), text(TIP)),
            ("paths".to_string(), OutValue::Texts(vec!["src".into(), "sub".into(), "root.txt".into()])),
            ("paths_changed".to_string(), OutValue::Bool(paths_changed)),
            ("moved".to_string(), OutValue::Bool(moved)),
        ],
    }
}

#[test]
fn each_refresh_outcome_has_its_exit_lines_next_and_fields() {
    let in_the_way = format!("{NOT_BROUGHT}: local changes are in the way at:");
    let failed = format!("{NOT_BROUGHT}: git could not read its index");
    let rows: Vec<(&str, bool, ZoneFolderOutcome, Report)> = vec![
        (
            "paths set and moved",
            true,
            ZoneFolderOutcome::Moved { folder: folder() },
            expected(
                ExitCode::Done,
                "refreshed",
                &[SET, "brought the zone folder /w/zone-core to e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"],
                "nothing to do",
                (true, true),
            ),
        ),
        (
            "paths set, already at the tip",
            true,
            ZoneFolderOutcome::AtTip { folder: folder() },
            expected(
                ExitCode::Done,
                "refreshed",
                &[SET, "the zone folder /w/zone-core is already at e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"],
                "nothing to do",
                (true, false),
            ),
        ),
        (
            "nothing to change",
            false,
            ZoneFolderOutcome::AtTip { folder: folder() },
            expected(
                ExitCode::Done,
                "already-refreshed",
                &[HELD, "the zone folder /w/zone-core is already at e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"],
                "nothing to do",
                (false, false),
            ),
        ),
        (
            "paths set, a path in the way",
            true,
            ZoneFolderOutcome::Refused { folder: folder(), why: FolderRefusal::InTheWay(vec!["src/file".into()]) },
            expected(
                ExitCode::Refused,
                "folder-not-moved",
                &[SET, &in_the_way, "src/file", "nothing in it changed"],
                "move each path listed as in the way out of its zone folder, or put it back to that folder's index \
                 copy, and leave the index as it is; then repeat the same command: mr folder refresh /w/zone-core \
                 --actor <name>",
                (true, false),
            ),
        ),
        (
            "git failed",
            false,
            ZoneFolderOutcome::Failed { folder: folder(), why: "git could not read its index".to_string() },
            expected(
                ExitCode::Failure,
                "folder-failed",
                &[HELD, &failed],
                "clear what stopped git in each zone folder that failed; then repeat the same command: mr folder \
                 refresh /w/zone-core --actor <name>",
                (false, false),
            ),
        ),
    ];
    let paths = ["src".to_string(), "sub".to_string(), "root.txt".to_string()];
    let tip = Oid::parse(TIP).expect("a valid oid");
    for (name, paths_changed, outcome, expected) in rows {
        assert_eq!(refresh_report("core", &folder(), &tip, &paths, paths_changed, &outcome), expected, "row {name}");
    }
}
