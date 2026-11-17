//! install_outcome: on installed and already-installed the PATH warning is the last line, printed just before
//! `next:`; a refusal or a failure comes back byte for byte, with no warning; no warning changes nothing.

use std::path::PathBuf;

use land::{GuardError, GuardState, HookState, LandRefs};

use super::{foreign_hook_report, guard_error_report, guard_installed_report, install_outcome};
use crate::report::{failure_report, render_plain, Report};

const WARNING: &str =
    "the first mr on PATH is /usr/local/bin/mr, not this Mortar: run Mortar as mortar (/b/target/debug/mortar)";

fn state() -> GuardState {
    GuardState {
        refs: Some(LandRefs {
            origin_branch: "release".to_string(),
            integration_branch: "develop".to_string(),
            remote: "upstream".to_string(),
        }),
        hooks_dir: PathBuf::from("/r/.git/hooks"),
        reference_transaction: HookState::Mortar,
        pre_push: HookState::Mortar,
    }
}

fn refusals_and_failures() -> Vec<Report> {
    vec![
        foreign_hook_report(&[PathBuf::from("/r/.git/hooks/pre-push")]),
        guard_error_report(&GuardError::ForeignHook { path: PathBuf::from("/r/.git/hooks/pre-push") }),
        guard_error_report(&GuardError::BadConfig {
            path: PathBuf::from("/r/.git/mortar/guard.json"),
            why: "it is not a JSON object".to_string(),
        }),
        failure_report("guard", "git could not name the repository's common dir"),
    ]
}

#[test]
fn installed_gets_the_warning_as_its_last_line_just_before_next() {
    let report = install_outcome(Ok(guard_installed_report(false, &state())), Some(WARNING.to_string()));
    assert_eq!(report.outcome, "installed");
    assert_eq!(report.lines.last().map(String::as_str), Some(WARNING));
    assert_eq!(
        render_plain(&report),
        "the guard is installed in /r/.git/hooks\n\
         it guards refs/heads/develop and refs/heads/release, and pushes of them to upstream\n\
         the first mr on PATH is /usr/local/bin/mr, not this Mortar: run Mortar as mortar (/b/target/debug/mortar)\n\
         next: nothing to do: git now refuses moves of develop and release that Mortar did not make\n"
    );
}

#[test]
fn already_installed_gets_the_warning_as_its_last_line_just_before_next() {
    let report = install_outcome(Ok(guard_installed_report(true, &state())), Some(WARNING.to_string()));
    assert_eq!(report.outcome, "already-installed");
    assert_eq!(report.lines.last().map(String::as_str), Some(WARNING));
    assert_eq!(
        render_plain(&report),
        "the guard is already installed in /r/.git/hooks; nothing was written\n\
         it guards refs/heads/develop and refs/heads/release, and pushes of them to upstream\n\
         the first mr on PATH is /usr/local/bin/mr, not this Mortar: run Mortar as mortar (/b/target/debug/mortar)\n\
         next: nothing to do: git now refuses moves of develop and release that Mortar did not make\n"
    );
}

#[test]
fn a_refusal_or_a_failure_comes_back_unchanged_with_no_warning() {
    for input in refusals_and_failures() {
        let plain = render_plain(&input);
        let report = install_outcome(Err(input.clone()), Some(WARNING.to_string()));
        assert_eq!(report, input);
        assert_eq!(render_plain(&report), plain);
        assert!(!render_plain(&report).contains("mr on PATH"), "{report:?}");
    }
}

#[test]
fn no_warning_changes_nothing() {
    for already in [false, true] {
        assert_eq!(
            install_outcome(Ok(guard_installed_report(already, &state())), None),
            guard_installed_report(already, &state())
        );
    }
    for input in refusals_and_failures() {
        assert_eq!(install_outcome(Err(input.clone()), None), input);
    }
}
