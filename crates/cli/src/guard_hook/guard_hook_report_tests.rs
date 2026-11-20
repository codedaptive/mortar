//! guard_hook_report: Allow is exit 0 with nothing to say; Refuse gives why, owner and next (exit 1).

use land::GuardVerdict;

use super::guard_hook_report;
use crate::report::{ExitCode, Report};

#[test]
fn allow_is_done_and_says_nothing() {
    assert_eq!(
        guard_hook_report(&GuardVerdict::Allow),
        Report {
            verb: "guard-hook".to_string(),
            exit: ExitCode::Done,
            outcome: "allowed".to_string(),
            lines: Vec::new(),
            owner: None,
            next: String::new(),
            fields: Vec::new(),
        }
    );
}

#[test]
fn refuse_is_refused_with_why_owner_and_next() {
    let verdict = GuardVerdict::Refuse {
        why: "only Mortar moves refs/heads/integration".to_string(),
        owner: "the landing surface".to_string(),
        next: "ask for a landing".to_string(),
    };
    assert_eq!(
        guard_hook_report(&verdict),
        Report {
            verb: "guard-hook".to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec!["only Mortar moves refs/heads/integration".to_string()],
            owner: Some("the landing surface".to_string()),
            next: "ask for a landing".to_string(),
            fields: Vec::new(),
        }
    );
}
