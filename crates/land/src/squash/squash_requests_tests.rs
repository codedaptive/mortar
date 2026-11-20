//! squash_requests: grouping by request, in first-commit order, skipping commits without a request.

use super::{squash_requests, SquashRequest};

fn commit(subject: &str, request: Option<&str>) -> (String, Option<String>) {
    (subject.to_string(), request.map(str::to_string))
}

#[test]
fn no_commits_give_no_requests() {
    assert_eq!(squash_requests(&[]), Vec::<SquashRequest>::new());
}

#[test]
fn commits_without_a_request_are_skipped() {
    let commits = [commit("wip", None), commit("Add a", Some("r1")), commit("fixup", None)];
    assert_eq!(
        squash_requests(&commits),
        vec![SquashRequest { request: "r1".to_string(), subjects: vec!["Add a".to_string()] }]
    );
}

#[test]
fn only_commits_without_a_request_give_no_requests() {
    assert_eq!(squash_requests(&[commit("wip", None), commit("more", None)]), Vec::<SquashRequest>::new());
}

#[test]
fn requests_are_grouped_in_first_commit_order_with_subjects_in_commit_order() {
    let commits = [
        commit("b one", Some("rb")),
        commit("a one", Some("ra")),
        commit("b two", Some("rb")),
        commit("c one", Some("rc")),
        commit("a two", Some("ra")),
    ];
    assert_eq!(
        squash_requests(&commits),
        vec![
            SquashRequest { request: "rb".to_string(), subjects: vec!["b one".to_string(), "b two".to_string()] },
            SquashRequest { request: "ra".to_string(), subjects: vec!["a one".to_string(), "a two".to_string()] },
            SquashRequest { request: "rc".to_string(), subjects: vec!["c one".to_string()] },
        ]
    );
}

#[test]
fn a_repeated_subject_is_kept_each_time() {
    let commits = [commit("same", Some("r1")), commit("same", Some("r1"))];
    assert_eq!(
        squash_requests(&commits),
        vec![SquashRequest { request: "r1".to_string(), subjects: vec!["same".to_string(), "same".to_string()] }]
    );
}
