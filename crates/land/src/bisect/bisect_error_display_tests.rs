//! Failure texts for OriginMissing, NotOnOrigin, OffTheLine and ProbeCouldNotRun. Culprit prints CulpritError's
//! text: culprit_error_display_tests cover NotALanding and tests/land_rit/displays.rs Trailers; Git's is not yet
//! written.

use gitio::Oid;

use super::BisectError;

fn a() -> Oid { Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap() }
fn b() -> Oid { Oid::parse("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap() }

#[test]
fn origin_missing_names_the_ref_and_says_to_name_a_local_branch() {
    let err = BisectError::OriginMissing { refname: "refs/heads/main".to_string() };
    assert_eq!(
        err.to_string(),
        "refs/heads/main does not exist in this repository; name an origin branch that exists here as a local \
         branch, then bisect again"
    );
}

#[test]
fn not_on_origin_names_the_commit_and_the_branch() {
    assert_eq!(BisectError::NotOnOrigin { failed: a(), origin_branch: "main".into() }.to_string(), "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa is not on main; give a failed commit that landed on main");
}

#[test]
fn off_the_line_names_both_commits() {
    assert_eq!(BisectError::OffTheLine { last_pass: a(), failed: b() }.to_string(), "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa is not on bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb's first-parent line, so the range names no landings; give a last passing commit that is a landing before bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
}

#[test]
fn probe_could_not_run_names_the_commit_and_the_error() {
    assert_eq!(BisectError::ProbeCouldNotRun { commit: a(), error: "spawn refused".into() }.to_string(), "the test command could not be run at aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa: spawn refused; fix the command, then bisect again");
}
