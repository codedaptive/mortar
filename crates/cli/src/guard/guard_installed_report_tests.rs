//! guard_installed_report: installed and already-installed, naming the hooks folder and the guarded branches.

use std::path::PathBuf;

use land::{GuardState, HookState, LandRefs};

use super::guard_installed_report;
use crate::report::{ExitCode, OutValue, Report};

fn text(value: &str) -> OutValue {
    OutValue::Text(value.to_string())
}

fn installed_state(refs: Option<LandRefs>) -> GuardState {
    GuardState {
        refs,
        hooks_dir: PathBuf::from("/r/.git/hooks"),
        reference_transaction: HookState::Mortar,
        pre_push: HookState::Mortar,
    }
}

fn develop_and_release() -> Option<LandRefs> {
    Some(LandRefs {
        origin_branch: "release".to_string(),
        integration_branch: "develop".to_string(),
        remote: "upstream".to_string(),
    })
}

#[test]
fn a_fresh_install() {
    assert_eq!(
        guard_installed_report(false, &installed_state(develop_and_release())),
        Report {
            verb: "guard".to_string(),
            exit: ExitCode::Done,
            outcome: "installed".to_string(),
            lines: vec![
                "the guard is installed in /r/.git/hooks".to_string(),
                "it guards refs/heads/develop and refs/heads/release, and pushes of them to upstream".to_string(),
            ],
            owner: None,
            next: "nothing to do: git now refuses moves of develop and release that Mortar did not make".to_string(),
            fields: vec![
                ("hooks_dir".to_string(), text("/r/.git/hooks")),
                ("reference_transaction".to_string(), text("mortar")),
                ("pre_push".to_string(), text("mortar")),
                ("origin_branch".to_string(), text("release")),
                ("integration_branch".to_string(), text("develop")),
                ("remote".to_string(), text("upstream")),
            ],
        }
    );
}

#[test]
fn a_repeat_is_already_installed_with_nothing_written() {
    let report = guard_installed_report(true, &installed_state(develop_and_release()));
    assert_eq!(report.exit, ExitCode::Done);
    assert_eq!(report.outcome, "already-installed");
    assert_eq!(
        report.lines,
        vec![
            "the guard is already installed in /r/.git/hooks; nothing was written".to_string(),
            "it guards refs/heads/develop and refs/heads/release, and pushes of them to upstream".to_string(),
        ]
    );
    assert_eq!(report.next, "nothing to do: git now refuses moves of develop and release that Mortar did not make");
}

#[test]
fn a_guard_json_gone_before_it_was_read_back_sends_the_reader_to_install() {
    let report = guard_installed_report(false, &installed_state(None));
    assert_eq!(report.outcome, "installed");
    assert_eq!(report.lines[1], "guard.json names no branches to guard");
    assert_eq!(report.next, "mr guard install --actor <name>");
    assert_eq!(report.fields[3], ("origin_branch".to_string(), OutValue::Null));
}
