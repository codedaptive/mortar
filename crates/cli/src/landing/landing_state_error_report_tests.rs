//! landing_state_error_report: Read, Write and Remove as failures naming the path, Corrupt refused with its reason.

use std::path::PathBuf;

use land::LandingStateError;

use super::landing_state_error_report;
use crate::report::{ExitCode, Report};

const PATH: &str = "/r/.git/mortar/landing.json";

fn failed(line: &str) -> Report {
    Report {
        verb: "back-out".to_string(),
        exit: ExitCode::Failure,
        outcome: "failed".to_string(),
        lines: vec![line.to_string()],
        owner: None,
        next: "repeat the same command".to_string(),
        fields: Vec::new(),
    }
}

#[test]
fn a_read_failure_names_the_path_not_the_io_error() {
    let error = LandingStateError::Read { path: PathBuf::from(PATH), source: std::io::Error::other("disk gone") };
    assert_eq!(
        landing_state_error_report("back-out", &error),
        failed("reading /r/.git/mortar/landing.json failed")
    );
}

#[test]
fn a_write_failure_names_the_path_not_the_io_error() {
    let error = LandingStateError::Write {
        path: PathBuf::from("/r/.git/mortar/landing.json.tmp"),
        source: std::io::Error::other("disk full"),
    };
    assert_eq!(
        landing_state_error_report("back-out", &error),
        failed("writing /r/.git/mortar/landing.json.tmp failed")
    );
}

#[test]
fn a_remove_failure_names_the_path_not_the_io_error() {
    let error = LandingStateError::Remove { path: PathBuf::from(PATH), source: std::io::Error::other("busy") };
    assert_eq!(
        landing_state_error_report("back-out", &error),
        failed("removing /r/.git/mortar/landing.json failed")
    );
}

#[test]
fn a_corrupt_state_is_refused_naming_the_path_and_reason() {
    let error = LandingStateError::Corrupt { path: PathBuf::from(PATH), reason: "line 2 has no origin".to_string() };
    assert_eq!(
        landing_state_error_report("publish", &error),
        Report {
            verb: "publish".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![
                "/r/.git/mortar/landing.json is not a landing state Mortar wrote: line 2 has no origin".to_string()
            ],
            owner: Some("the landing in flight".to_string()),
            next: "mend or remove /r/.git/mortar/landing.json by hand, then repeat the same command".to_string(),
            fields: Vec::new(),
        }
    );
}
