//! sweep_report: exit 0; "swept" naming each deleted ref, each removed build dir and the bytes freed, or
//! "nothing-to-sweep"; without --idle-days (build None) it says the build dirs were not swept.

use std::path::PathBuf;

use folders::SweepBuildOutcome;

use super::sweep_report;
use crate::report::{ExitCode, OutValue, Report};

fn deleted() -> Vec<String> {
    vec!["refs/mortar/replayed/gone/abc".to_string(), "refs/t3/old".to_string()]
}

#[test]
fn deleted_refs_and_a_build_outcome_are_each_named() {
    let build = SweepBuildOutcome { removed: vec![PathBuf::from("/w/fix-1/target")], freed_bytes: 2048 };
    assert_eq!(
        sweep_report(&deleted(), Some(&build)),
        Report {
            verb: "sweep".to_string(),
            exit: ExitCode::Done,
            outcome: "swept".to_string(),
            lines: vec![
                "deleted refs/mortar/replayed/gone/abc".to_string(),
                "deleted refs/t3/old".to_string(),
                "removed build dir /w/fix-1/target".to_string(),
                "freed 2048 bytes of build dirs".to_string(),
            ],
            owner: None,
            next: "nothing to do".to_string(),
            fields: vec![
                (
                    "deleted".to_string(),
                    OutValue::Texts(vec!["refs/mortar/replayed/gone/abc".to_string(), "refs/t3/old".to_string()])
                ),
                ("removed".to_string(), OutValue::Texts(vec!["/w/fix-1/target".to_string()])),
                ("freed_bytes".to_string(), OutValue::Number(2048)),
            ],
        }
    );
}

#[test]
fn nothing_swept_is_nothing_to_sweep() {
    let build = SweepBuildOutcome { removed: Vec::new(), freed_bytes: 0 };
    assert_eq!(
        sweep_report(&[], Some(&build)),
        Report {
            verb: "sweep".to_string(),
            exit: ExitCode::Done,
            outcome: "nothing-to-sweep".to_string(),
            lines: vec!["nothing to sweep".to_string(), "freed 0 bytes of build dirs".to_string()],
            owner: None,
            next: "nothing to do".to_string(),
            fields: vec![
                ("deleted".to_string(), OutValue::Texts(Vec::new())),
                ("removed".to_string(), OutValue::Texts(Vec::new())),
                ("freed_bytes".to_string(), OutValue::Number(0)),
            ],
        }
    );
}

#[test]
fn without_idle_days_the_build_dirs_are_not_swept() {
    assert_eq!(
        sweep_report(&deleted(), None),
        Report {
            verb: "sweep".to_string(),
            exit: ExitCode::Done,
            outcome: "swept".to_string(),
            lines: vec![
                "deleted refs/mortar/replayed/gone/abc".to_string(),
                "deleted refs/t3/old".to_string(),
                "build dirs not swept: no --idle-days".to_string(),
            ],
            owner: None,
            next: "nothing to do".to_string(),
            fields: vec![
                (
                    "deleted".to_string(),
                    OutValue::Texts(vec!["refs/mortar/replayed/gone/abc".to_string(), "refs/t3/old".to_string()])
                ),
                ("removed".to_string(), OutValue::Texts(Vec::new())),
                ("freed_bytes".to_string(), OutValue::Null),
            ],
        }
    );
}

#[test]
fn no_refs_and_no_idle_days_is_nothing_to_sweep() {
    let report = sweep_report(&[], None);
    assert_eq!(report.exit as i32, 0);
    assert_eq!(report.outcome, "nothing-to-sweep");
    assert_eq!(
        report.lines,
        vec!["nothing to sweep".to_string(), "build dirs not swept: no --idle-days".to_string()]
    );
}
