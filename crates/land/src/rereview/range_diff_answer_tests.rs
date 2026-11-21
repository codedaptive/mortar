//! range_diff_answer: a failed patch-ID read is the error, the reviewed commit's before the new one's; with both
//! reads Ok the text passes through and same_patch is true only for two equal IDs, never for an empty change.

use gitio::{GitError, Oid, PatchId};

use super::{range_diff_answer, RereviewAnswer, RereviewError, RereviewPlan};

const PATCH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const PATCH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const TEXT: &str = "1:  1111111 ! 1:  2222222 add f\n";

fn patch(text: &str) -> Option<PatchId> {
    Some(PatchId::parse(text).unwrap())
}

fn git_error(stderr: &str) -> GitError {
    GitError { args: vec!["patch-id".to_string()], code: Some(128), stderr: stderr.to_string() }
}

fn range_diff() -> RereviewPlan {
    RereviewPlan::RangeDiff {
        reviewed_base: Oid::parse("1111111111111111111111111111111111111111").unwrap(),
        new_base: Oid::parse("2222222222222222222222222222222222222222").unwrap(),
    }
}

fn answer(
    reviewed: Result<Option<PatchId>, GitError>,
    new: Result<Option<PatchId>, GitError>,
) -> Result<RereviewAnswer, RereviewError> {
    range_diff_answer(&range_diff(), TEXT.to_string(), reviewed, new)
}

fn inner_git_error(result: Result<RereviewAnswer, RereviewError>) -> GitError {
    match result {
        Err(RereviewError::Git(inner)) => inner,
        other => panic!("expected a Git error, got {other:?}"),
    }
}

#[test]
fn reviewed_read_failing_is_the_error() {
    let result = answer(Err(git_error("reviewed failed")), Ok(patch(PATCH_A)));
    assert_eq!(inner_git_error(result), git_error("reviewed failed"));
}

#[test]
fn new_read_failing_is_the_error() {
    assert_eq!(inner_git_error(answer(Ok(patch(PATCH_A)), Err(git_error("new failed")))), git_error("new failed"));
}

#[test]
fn both_reads_failing_is_the_reviewed_error() {
    let result = answer(Err(git_error("reviewed failed")), Err(git_error("new failed")));
    assert_eq!(inner_git_error(result), git_error("reviewed failed"));
}

#[test]
fn equal_ids_are_the_same_patch_with_the_text_unchanged() {
    let expected = RereviewAnswer { text: "1:  1111111 ! 1:  2222222 add f\n".to_string(), same_patch: true };
    assert_eq!(answer(Ok(patch(PATCH_A)), Ok(patch(PATCH_A))).unwrap(), expected);
}

#[test]
fn different_ids_are_not_the_same_patch() {
    assert!(!answer(Ok(patch(PATCH_A)), Ok(patch(PATCH_B))).unwrap().same_patch);
}

#[test]
fn an_empty_reviewed_change_is_an_answer_not_the_same_patch() {
    assert!(!answer(Ok(None), Ok(patch(PATCH_A))).unwrap().same_patch);
}

#[test]
fn an_empty_new_change_is_an_answer_not_the_same_patch() {
    assert!(!answer(Ok(patch(PATCH_A)), Ok(None)).unwrap().same_patch);
}
