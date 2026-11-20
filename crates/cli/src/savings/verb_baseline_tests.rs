//! verb_baseline: status, rereview, would-merge and overlap name their git read; every other verb has none.

use super::verb_baseline;

#[test]
fn status_replaces_git_status_and_diff_head() {
    assert_eq!(verb_baseline("status"), Some("git status and git diff HEAD over this folder"));
}

#[test]
fn rereview_replaces_the_diff_between_reviews() {
    assert_eq!(verb_baseline("rereview"), Some("git diff <reviewed> <new>"));
}

#[test]
fn would_merge_replaces_the_log_of_its_range() {
    assert_eq!(verb_baseline("would-merge"), Some("git log -p <base>..<commit>"));
}

#[test]
fn overlap_replaces_the_log_of_each_range() {
    assert_eq!(verb_baseline("overlap"), Some("git log -p of each request's range"));
}

#[test]
fn other_verbs_have_none() {
    for verb in ["log", "save", "join", "help", "land", "Status", ""] {
        assert_eq!(verb_baseline(verb), None, "{verb}");
    }
}
