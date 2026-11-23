//! stopped_run_refusal: refused naming the commit the paths were staged on and the tip, owned by the zone; next
//! restores the paths from that commit, refreshes the folder, then runs mr commit-files again.

use std::path::{Path, PathBuf};

use gitio::Oid;

use super::stopped_run_refusal;
use crate::report::{ExitCode, OutValue, Report};

const STAGED_ON: &str = "1111111111111111111111111111111111111111";
const TIP: &str = "2222222222222222222222222222222222222222";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn the_refusal_names_the_old_commit_the_tip_and_the_restore() {
    for (paths, next) in [
        (
            &["src/a.rs"][..],
            "git restore --staged --source=1111111111111111111111111111111111111111 -- src/a.rs, then mr folder \
             refresh /w/core --actor <name> (if the refresh refuses on a path, move that file aside and put \
             it back after), then mr commit-files again",
        ),
        (
            &["src/a.rs", "b/c"],
            "git restore --staged --source=1111111111111111111111111111111111111111 -- src/a.rs b/c, then mr folder \
             refresh /w/core --actor <name> (if the refresh refuses on a path, move that file aside and put \
             it back after), then mr commit-files again",
        ),
    ] {
        let paths: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
        let expected = Report {
            verb: "commit-files".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "a stopped mr commit-files left the named paths staged on 1111111111111111111111111111111111111111; \
                 zone/core has moved to 2222222222222222222222222222222222222222 since"
                    .to_string(),
                "nothing was written and no ref moved".to_string(),
            ],
            owner: Some("zone core".to_string()),
            next: next.to_string(),
            fields: vec![
                ("zone".to_string(), OutValue::Text("core".to_string())),
                ("tip".to_string(), OutValue::Text(TIP.to_string())),
            ],
        };
        let actual = stopped_run_refusal("core", Path::new("/w/core"), &oid(STAGED_ON), &oid(TIP), &paths);
        assert_eq!(actual, expected, "{paths:?}");
    }
}
