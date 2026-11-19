use gitio::{GitError, Version};

use super::git_floor_report;
use crate::report::{self, ExitCode, Report};

#[test]
fn below_floor_is_refused_with_the_found_version_and_install_step() {
    assert_eq!(
        Version::FLOOR,
        Version { major: 2, minor: 46, patch: 0 },
        "a floor move means updating this report pin"
    );
    let error = Version { major: 2, minor: 39, patch: 5 }.require_floor().unwrap_err();
    assert_eq!(
        git_floor_report("status", &error),
        Report {
            verb: "status".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![error.stderr.clone()],
            owner: Some("this machine's git".to_string()),
            next: "install git 2.46 or newer, then repeat the command".to_string(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn unreadable_version_output_is_refused_with_the_below_floor_install_step() {
    let error = Version::parse(b"not git at all").unwrap_err();
    let below_floor = Version { major: 2, minor: 39, patch: 5 }.require_floor().unwrap_err();
    assert_eq!(
        git_floor_report("join", &error),
        Report {
            verb: "join".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![error.stderr.clone()],
            owner: Some("this machine's git".to_string()),
            next: git_floor_report("join", &below_floor).next,
            fields: Vec::new(),
        }
    );
}

#[test]
fn below_floor_next_step_names_floor_without_patch() {
    let floor = Version::FLOOR;
    let error = Version { major: 2, minor: 39, patch: 5 }.require_floor().unwrap_err();
    let next = git_floor_report("status", &error).next;
    // The report pin carries today's value; this ties the text to FLOOR if the floor moves but the code does not.
    assert!(next.contains(&format!("git {}.{} or newer", floor.major, floor.minor)));
    assert!(!next.contains(&format!("{}.{}.{}", floor.major, floor.minor, floor.patch)));
}

#[test]
fn git_version_spawn_failure_is_a_failure_report() {
    let error = GitError {
        args: vec!["version".to_string()],
        code: None,
        stderr: "could not run git: No such file or directory (os error 2)".to_string(),
    };
    assert_eq!(git_floor_report("status", &error), report::failure_report("status", &error.to_string()));
}

#[test]
fn nonzero_git_version_is_a_failure_report() {
    let error = GitError {
        args: vec!["version".to_string()],
        code: Some(1),
        stderr: "git version failed".to_string(),
    };
    assert_eq!(git_floor_report("join", &error), report::failure_report("join", &error.to_string()));
}
