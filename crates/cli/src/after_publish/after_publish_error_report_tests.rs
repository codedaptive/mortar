//! after_publish_error_report: every AfterPublishError arm, and JoinError's arm by arm, in the cli's own words; each
//! refusal's and failure's second line says the zones before it were already moved and no zone folder was brought to
//! its tip, and no report says that nothing half-done is left.

use gitio::{GitError, Oid};
use join::JoinError;
use land::{AfterPublishError, LandError, SquashError};

use super::after_publish_error_report;
use crate::report::{ExitCode, Report};

const ORIGIN: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
const TIP: &str = "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391";
const COMMIT: &str = "9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e";
const SECOND: &str = "any zone named before it was already moved; no zone folder was brought to its tip";
const ONE_PARENT_NEXT: &str =
    "rewrite the zone's commits as one-parent commits with author dates, then repeat the same command";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn report(error: &AfterPublishError) -> Report {
    after_publish_error_report(error, "stage")
}

fn git_error() -> GitError {
    GitError::refusal(vec!["replay".to_string()], "fatal: gitio text 5521")
}

fn refused(lines: &[&str], owner: &str, next: &str) -> Report {
    Report {
        verb: "after-publish".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: lines.iter().map(ToString::to_string).collect(),
        owner: Some(owner.to_string()),
        next: next.to_string(),
        fields: Vec::new(),
    }
}

const HALF_DONE: &str = "nothing half-done is left";

fn failed(line: &str) -> Report {
    Report {
        verb: "after-publish".to_string(),
        exit: ExitCode::Failure,
        outcome: "failed".to_string(),
        lines: vec![line.to_string(), SECOND.to_string()],
        owner: None,
        next: "repeat the same command".to_string(),
        fields: Vec::new(),
    }
}

#[test]
fn git_is_a_failure_in_the_clis_words() {
    let report = report(&AfterPublishError::Git(git_error()));
    assert_eq!(
        report,
        failed("git failed while reading or replaying a zone")
    );
}

#[test]
fn a_squash_error_is_worded_as_the_landing_verbs_word_it() {
    let error = AfterPublishError::Squash(SquashError::RefMissing { refname: "refs/heads/zone/cli".to_string() });
    assert_eq!(
        report(&error),
        refused(
            &["refs/heads/zone/cli does not exist; nothing was written and no ref moved", SECOND],
            "the repository's branches",
            "create refs/heads/zone/cli, then repeat the same command",
        )
    );
}

#[test]
fn a_land_error_is_worded_as_the_landing_verbs_word_it_with_the_integration_branch() {
    let error = AfterPublishError::Land(LandError::NoLandingState { origin: oid(ORIGIN), integration: oid(TIP) });
    assert_eq!(
        report(&error),
        refused(
            &["integration is at e69de29bb2d1d6434b8b29ae775ad8c2e48c5391, not at origin \
               4b825dc642cb6eb9a060e54bf8d69288fbee4904, and no landing state names a landing in flight",
              SECOND],
            "refs/heads/stage",
            "find the op that moved it with mortar log --ref refs/heads/stage, then mortar restore <op id> \
             refs/heads/stage --actor <name>",
        )
    );
}

#[test]
fn a_join_git_error_is_a_failure_in_the_clis_words() {
    let report = report(&AfterPublishError::Join(JoinError::Git(git_error())));
    assert_eq!(
        report,
        failed("git failed while replaying a zone onto origin")
    );
}

#[test]
fn a_missing_onto_commit_is_refused_naming_it() {
    let error = AfterPublishError::Join(JoinError::OntoMissing(oid(ORIGIN)));
    assert_eq!(
        report(&error),
        refused(
            &[
                "origin's commit 4b825dc642cb6eb9a060e54bf8d69288fbee4904 does not exist here, so no zone can be \
                 replayed onto it",
                SECOND,
            ],
            "the repository's objects",
            "fetch or restore commit 4b825dc642cb6eb9a060e54bf8d69288fbee4904, then repeat the same command",
        )
    );
}

#[test]
fn a_missing_tip_is_refused_naming_the_ref() {
    let error = AfterPublishError::Join(JoinError::TipMissing("refs/heads/zone/land".to_string()));
    assert_eq!(
        report(&error),
        refused(
            &["refs/heads/zone/land does not exist, so its zone cannot be replayed", SECOND],
            "the repository's branches",
            "create refs/heads/zone/land, then repeat the same command",
        )
    );
}

#[test]
fn a_merge_commit_is_refused_to_the_zones_history() {
    let error = AfterPublishError::Join(JoinError::MergeCommit(oid(COMMIT)));
    assert_eq!(
        report(&error),
        refused(
            &["zone commit 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e is a merge, so replay cannot take it", SECOND],
            "the zone's history",
            ONE_PARENT_NEXT,
        )
    );
}

#[test]
fn a_root_commit_is_refused_to_the_zones_history() {
    let error = AfterPublishError::Join(JoinError::RootCommit(oid(COMMIT)));
    assert_eq!(
        report(&error),
        refused(
            &["zone commit 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e has no parent, so replay cannot take it", SECOND],
            "the zone's history",
            ONE_PARENT_NEXT,
        )
    );
}

#[test]
fn an_empty_author_date_is_refused_to_the_zones_history() {
    let error = AfterPublishError::Join(JoinError::EmptyAuthorDate(oid(COMMIT)));
    assert_eq!(
        report(&error),
        refused(
            &[
                "zone commit 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e has an empty author date, so replay cannot take \
                 it",
                SECOND,
            ],
            "the zone's history",
            ONE_PARENT_NEXT,
        )
    );
}

#[test]
fn a_tip_not_replayed_is_a_failure_naming_the_ref() {
    let error = AfterPublishError::Join(JoinError::TipNotReplayed("refs/heads/zone/join".to_string()));
    assert_eq!(
        report(&error),
        failed("git's replay gave no update for refs/heads/zone/join")
    );
}

#[test]
fn an_unrelated_zone_is_refused_naming_its_tip_and_origin() {
    let error = AfterPublishError::Unrelated { zone: "land".to_string(), origin: oid(ORIGIN), tip: oid(TIP) };
    assert_eq!(
        report(&error),
        refused(
            &[
                "zone land at e69de29bb2d1d6434b8b29ae775ad8c2e48c5391 shares no history with origin \
                 4b825dc642cb6eb9a060e54bf8d69288fbee4904",
                SECOND,
            ],
            "zone land",
            "rebuild the zone branch from origin, then repeat the same command",
        )
    );
}

#[test]
fn no_squash_of_is_refused_asking_for_the_published_zone() {
    let error = AfterPublishError::NoSquashOf { zone: "cli".to_string() };
    assert_eq!(
        report(&error),
        refused(
            &[
                "no squash of zone cli on origin's line carries a Mortar-Squash-Of trailer, so the squash published \
                 from it cannot be found",
                SECOND,
            ],
            "zone cli's squashes",
            "name the zone whose squash was published: mortar after-publish <zone> [--move <zone>]... --actor <name>",
        )
    );
}

#[test]
fn a_squash_source_the_zone_does_not_hold_is_refused_naming_the_zone_and_the_source() {
    let error = AfterPublishError::SquashOfNotInZone { zone: "cli".to_string(), squash_of: oid(COMMIT) };
    assert_eq!(
        report(&error),
        refused(
            &[
                "zone cli's published squash was made from 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e, which zone cli \
                 does not hold, itself or as a recorded replay",
                SECOND,
            ],
            "zone cli",
            "restore refs/heads/zone/cli to a commit that holds 9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e (find it \
             with mortar log --ref refs/heads/zone/cli, then mortar restore <op id> refs/heads/zone/cli --actor \
             <name>), then repeat the same command",
        )
    );
}

#[test]
fn no_arm_prints_gitios_text() {
    for error in [
        AfterPublishError::Git(git_error()),
        AfterPublishError::Join(JoinError::Git(git_error())),
        AfterPublishError::Squash(SquashError::Git(git_error())),
        AfterPublishError::Land(LandError::Git(git_error())),
    ] {
        let printed = format!("{:?}", report(&error));
        assert!(!printed.contains("5521"), "{printed}");
    }
}

fn assert_failure_without_half_done(error: &AfterPublishError) {
    let report = report(error);
    assert_eq!(report.exit, ExitCode::Failure, "{report:?}");
    assert!(!report.lines.iter().any(|line| line == HALF_DONE), "{report:?}");
    assert!(report.lines.iter().any(|line| line == SECOND), "{report:?}");
}

#[test]
fn a_git_failure_keeps_second_and_never_says_nothing_half_done() {
    assert_failure_without_half_done(&AfterPublishError::Git(git_error()));
}

#[test]
fn a_join_git_failure_keeps_second_and_never_says_nothing_half_done() {
    assert_failure_without_half_done(&AfterPublishError::Join(JoinError::Git(git_error())));
}

#[test]
fn a_tip_not_replayed_keeps_second_and_never_says_nothing_half_done() {
    assert_failure_without_half_done(&AfterPublishError::Join(JoinError::TipNotReplayed(
        "refs/heads/zone/join".to_string(),
    )));
}

#[test]
fn a_squash_git_failure_keeps_second_and_never_says_nothing_half_done() {
    assert_failure_without_half_done(&AfterPublishError::Squash(SquashError::Git(git_error())));
}

#[test]
fn a_land_git_failure_keeps_second_and_never_says_nothing_half_done() {
    assert_failure_without_half_done(&AfterPublishError::Land(LandError::Git(git_error())));
}
