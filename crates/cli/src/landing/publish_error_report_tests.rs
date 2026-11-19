//! publish_error_report: every PublishError variant in the cli's own words, as "publish", and each PushRefusal's
//! reason.

use std::path::PathBuf;

use gitio::{GitError, Oid, PushRefusal};
use land::{LandError, LandRefs, LandingStateError, PublishError, SquashError};
use oplog::{OpPhase, OplogError};

use super::publish_error_report;
use crate::report::{ExitCode, Report};

fn refs() -> LandRefs {
    LandRefs {
        origin_branch: "main".to_string(),
        integration_branch: "integ".to_string(),
        remote: "upstream".to_string(),
    }
}

fn refused(lines: &[&str], owner: &str, next: &str) -> Report {
    Report {
        verb: "publish".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: lines.iter().map(|line| line.to_string()).collect(),
        owner: Some(owner.to_string()),
        next: next.to_string(),
        fields: Vec::new(),
    }
}

fn failed(line: &str) -> Report {
    Report {
        verb: "publish".to_string(),
        exit: ExitCode::Failure,
        outcome: "failed".to_string(),
        lines: vec![line.to_string()],
        owner: None,
        next: "repeat the same command".to_string(),
        fields: Vec::new(),
    }
}

fn push_refused(refusal: PushRefusal) -> Report {
    publish_error_report(&PublishError::PushRefused { refname: "refs/heads/main".to_string(), refusal }, &refs())
}

fn push_refused_report(why: &str) -> Report {
    refused(
        &[
            &format!("the remote upstream refused the push of refs/heads/main: {why}"),
            "the remote's refs/heads/main is still at the landing's starting origin",
        ],
        "the remote upstream",
        "clear what the remote refused, then mr publish --actor <name>",
    )
}

#[test]
fn a_squash_error_is_squashs_report_as_publish() {
    let error = PublishError::Squash(SquashError::RefMissing { refname: "refs/remotes/upstream/main".to_string() });
    assert_eq!(
        publish_error_report(&error, &refs()),
        refused(
            &["refs/remotes/upstream/main does not exist; nothing was written and no ref moved"],
            "the repository's branches",
            "create refs/remotes/upstream/main, then repeat the same command",
        )
    );
}

#[test]
fn a_land_error_names_the_configured_integration_branch() {
    let a = Oid::parse("1111111111111111111111111111111111111111").unwrap();
    let b = Oid::parse("2222222222222222222222222222222222222222").unwrap();
    let error = PublishError::Land(LandError::IntegrationElsewhere { expected: a, actual: b });
    assert_eq!(
        publish_error_report(&error, &refs()),
        refused(
            &["integration is at 2222222222222222222222222222222222222222, at neither origin nor the landed commit \
               1111111111111111111111111111111111111111"],
            "refs/heads/integ",
            "find the op that moved it with mr log --ref refs/heads/integ, then mr restore <op id> \
             refs/heads/integ --actor <name>",
        )
    );
}

#[test]
fn a_state_error_is_the_landing_states_report_as_publish() {
    let path = PathBuf::from("/r/.git/mortar/landing.json");
    let error = PublishError::State(LandingStateError::Remove { path, source: std::io::Error::other("busy") });
    assert_eq!(publish_error_report(&error, &refs()), failed("removing /r/.git/mortar/landing.json failed"));
}

#[test]
fn git_and_oplog_are_failures_in_the_clis_words() {
    let git = PublishError::Git(GitError::refusal(Vec::new(), "gitio text"));
    assert_eq!(
        publish_error_report(&git, &refs()),
        failed("git failed while reading or pushing to the remote, or reading a ref")
    );
    let oplog = PublishError::Oplog(OplogError::Malformed { line: None, reason: "oplog text".to_string() });
    assert_eq!(publish_error_report(&oplog, &refs()), failed("the op log could not be written"));
}

#[test]
fn a_stale_lease_refusal_says_the_value_was_not_the_one_leased() {
    assert_eq!(push_refused(PushRefusal::StaleLease), push_refused_report("its value was not the one leased"));
}

#[test]
fn a_non_fast_forward_refusal_says_so() {
    assert_eq!(push_refused(PushRefusal::NonFastForward), push_refused_report("it is not a fast-forward"));
}

#[test]
fn a_hook_refusal_says_a_remote_hook_declined_it() {
    assert_eq!(push_refused(PushRefusal::Hook), push_refused_report("a hook on the remote declined it"));
}

#[test]
fn another_refusal_quotes_gits_summary() {
    assert_eq!(
        push_refused(PushRefusal::Other("[rejected] (fetch first)".to_string())),
        push_refused_report("git's summary was [rejected] (fetch first)")
    );
}

#[test]
fn a_missing_remote_branch_is_refused_naming_remote_and_branch() {
    let error = PublishError::RemoteMissing { remote: "upstream".to_string(), refname: "refs/heads/main".to_string() };
    assert_eq!(
        publish_error_report(&error, &refs()),
        refused(
            &["upstream has no refs/heads/main to publish onto; nothing was pushed"],
            "the remote upstream",
            "create refs/heads/main on upstream, then mr publish --actor <name>",
        )
    );
}

#[test]
fn a_push_intent_in_the_log_names_its_op_id_and_phase() {
    let cases = [(OpPhase::Intent, "open"), (OpPhase::Done, "done"), (OpPhase::Failed, "failed")];
    for (phase, words) in cases {
        let error = PublishError::PushIntent { op_id: "op-3".to_string(), phase };
        assert_eq!(
            publish_error_report(&error, &refs()),
            refused(
                &[&format!(
                    "the push intent op-3 for this move is already in the op log ({words}); nothing was pushed"
                )],
                "the op log",
                "mr publish --actor <name>; a repeat settles the open intent",
            ),
            "{phase:?}"
        );
    }
}
