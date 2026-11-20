//! Rereview reports same-patch squashes and leaves the review decision to the caller.
//! Drives the mortar binary against throwaway repositories.

use super::run_rereview_tests::{commit_on, mortar, repo, squash_in_setup};
use crate::json::check;

#[test]
fn a_remake_on_a_moved_origin_with_the_same_change_is_same_patch() {
    let (_repo, root) = repo();
    commit_on(&root, "zone/a", "src/a/x", "zone edit of x1\nx2\nx3\nx4\nx5\nx6\nx7\nx8\n");
    let reviewed = squash_in_setup(&root, "a");
    commit_on(&root, "main", "src/b/y", "origin moved\n");
    let new = squash_in_setup(&root, "a");
    assert_eq!(new.change_id, reviewed.change_id);
    assert_ne!(new.commit, reviewed.commit);

    let output = mortar(&root, &["--actor", "accept", "--json", "rereview", reviewed.commit.as_str(), new.commit.as_str()]);
    let report = check(output, "rereview", 0, "same-patch");
    assert!(report.flag("same_patch"));
    assert_eq!(report.text("reviewed"), reviewed.commit.as_str());
    assert_eq!(report.text("new"), new.commit.as_str());
    assert_eq!(report.text("next"), format!(
        "{} carries the same patch as {}; whether the review of {} still holds on the new base is the caller's to decide",
        new.commit.as_str(), reviewed.commit.as_str(), reviewed.commit.as_str()
    ));
    // git's range-diff pairs the two squashes' commits as equal ("1:  <old> = 1:  <new> ..."): land's text, shown.
    let lines = report.lines();
    assert!(lines.contains(" = "), "{lines}");
    assert!(!lines.contains("the range-diff is empty"), "{lines}");
}

#[test]
fn identical_commits_are_same_patch() {
    let (_repo, root) = repo();
    commit_on(&root, "zone/a", "src/a/x", "zone edit of x1\nx2\nx3\nx4\nx5\nx6\nx7\nx8\n");
    let reviewed = squash_in_setup(&root, "a");
    let id = reviewed.commit.as_str();

    let output = mortar(&root, &["--actor", "accept", "--json", "rereview", id, id]);
    let report = check(output, "rereview", 0, "same-patch");
    assert!(report.flag("same_patch"));
    assert_eq!(report.text("reviewed"), id);
    assert_eq!(report.text("new"), id);
    assert_eq!(report.text("next"), format!(
        "{id} carries the same patch as {id}; whether the review of {id} still holds on the new base is the caller's to decide"
    ));
    assert!(report.lines().contains("nothing changed"));
}
