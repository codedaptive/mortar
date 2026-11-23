//! zone_off_tip_refusal: AtTip is not refused; From is refused naming the commit the folder stands on, with a refresh
//! as the next step; NoMatch is refused as a folder whose commit cannot be told, with a git diff against the zone
//! branch and then unstaging as the next step.

use std::path::Path;

use gitio::Oid;

use super::zone_off_tip_refusal;
use crate::report::{ExitCode, OutValue, Report};
use crate::zone_folder_move::ZoneOld;

const TIP: &str = "1111111111111111111111111111111111111111";
const OLD: &str = "2222222222222222222222222222222222222222";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn refused(first: &str, next: &str) -> Option<Report> {
    Some(Report {
        verb: "commit-files".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![first.to_string(), "nothing was written and no ref moved".to_string()],
        owner: Some("zone core".to_string()),
        next: next.to_string(),
        fields: vec![
            ("zone".to_string(), OutValue::Text("core".to_string())),
            ("tip".to_string(), OutValue::Text(TIP.to_string())),
        ],
    })
}

#[test]
fn only_a_folder_at_the_tip_is_let_through() {
    for (old, expected) in [
        (ZoneOld::AtTip, None),
        (
            ZoneOld::From(oid(OLD)),
            refused(
                "the zone folder /w/core stands on 2222222222222222222222222222222222222222, not on zone/core's tip \
                 1111111111111111111111111111111111111111",
                "mr folder refresh /w/core --actor <name>, then mr commit-files again",
            ),
        ),
        (
            ZoneOld::NoMatch,
            refused(
                "the zone folder /w/core's index holds staged changes at no commit zone/core has held, so the commit \
                 it stands on cannot be told",
                "git diff zone/core -- <path>... shows what the commit would change on the tip; if that is only \
                 your change, git restore --staged -- <path>... and run mr commit-files again",
            ),
        ),
    ] {
        assert_eq!(zone_off_tip_refusal("core", Path::new("/w/core"), &oid(TIP), &old), expected, "{old:?}");
    }
}
