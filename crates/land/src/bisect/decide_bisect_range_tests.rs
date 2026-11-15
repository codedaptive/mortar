//! decide_bisect_range over a scripted first-parent reader: the failed commit off origin refused with one read, the
//! last pass off the line refused after two, both ends on the line answering the reader's commits, and a reader
//! error returned with no read after it.

use gitio::Oid;

use crate::LandRefs;

use super::{decide_bisect_range, BisectError};

const TIP: &str = "1111111111111111111111111111111111111111";
const LAST_PASS: &str = "2222222222222222222222222222222222222222";
const FAILED: &str = "3333333333333333333333333333333333333333";
const MIDDLE: &str = "4444444444444444444444444444444444444444";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn refs() -> LandRefs {
    LandRefs { origin_branch: "main".into(), integration_branch: "integration".into(), remote: "origin".into() }
}

type Answer = Result<Option<Vec<Oid>>, BisectError>;

/// Runs decide_bisect_range with a reader that gives `answers` in order, returning its result and every
/// (base, tip) pair the reader was asked, in order.
fn run(answers: Vec<Answer>) -> (Result<Vec<Oid>, BisectError>, Vec<(Oid, Oid)>) {
    let mut answers = answers.into_iter();
    let mut asked = Vec::new();
    let result = decide_bisect_range(&refs(), &oid(TIP), &oid(LAST_PASS), &oid(FAILED), |base, tip| {
        asked.push((base.clone(), tip.clone()));
        answers.next().expect("the reader was asked more often than scripted")
    });
    (result, asked)
}

#[test]
fn a_failed_commit_off_origin_is_not_on_origin_after_one_read() {
    let (result, asked) = run(vec![Ok(None), Ok(Some(vec![oid(FAILED)]))]);
    assert!(
        matches!(&result, Err(BisectError::NotOnOrigin { failed, origin_branch })
            if *failed == oid(FAILED) && origin_branch == "main"),
        "{result:?}"
    );
    assert_eq!(asked, vec![(oid(FAILED), oid(TIP))]);
}

#[test]
fn a_last_pass_off_the_line_is_off_the_line_after_two_reads() {
    let (result, asked) = run(vec![Ok(Some(vec![oid(TIP)])), Ok(None)]);
    assert!(
        matches!(&result, Err(BisectError::OffTheLine { last_pass, failed })
            if *last_pass == oid(LAST_PASS) && *failed == oid(FAILED)),
        "{result:?}"
    );
    assert_eq!(asked, vec![(oid(FAILED), oid(TIP)), (oid(LAST_PASS), oid(FAILED))]);
}

#[test]
fn both_ends_on_the_line_answer_the_commits_from_last_pass_to_failed() {
    let (result, asked) = run(vec![Ok(Some(vec![oid(TIP)])), Ok(Some(vec![oid(MIDDLE), oid(FAILED)]))]);
    assert_eq!(result.unwrap(), vec![oid(MIDDLE), oid(FAILED)]);
    assert_eq!(asked, vec![(oid(FAILED), oid(TIP)), (oid(LAST_PASS), oid(FAILED))]);
}

#[test]
fn a_reader_error_on_the_first_read_is_returned_with_no_second_read() {
    let (result, asked) = run(vec![
        Err(BisectError::OriginMissing { refname: "refs/heads/main".into() }),
        Ok(Some(vec![oid(FAILED)])),
    ]);
    assert!(
        matches!(&result, Err(BisectError::OriginMissing { refname }) if refname == "refs/heads/main"),
        "{result:?}"
    );
    assert_eq!(asked, vec![(oid(FAILED), oid(TIP))]);
}

#[test]
fn a_reader_error_on_the_second_read_is_returned() {
    let (result, asked) = run(vec![
        Ok(Some(vec![oid(TIP)])),
        Err(BisectError::ProbeCouldNotRun { commit: oid(MIDDLE), error: "no such file".into() }),
    ]);
    assert!(
        matches!(&result, Err(BisectError::ProbeCouldNotRun { commit, error })
            if *commit == oid(MIDDLE) && error == "no such file"),
        "{result:?}"
    );
    assert_eq!(asked, vec![(oid(FAILED), oid(TIP)), (oid(LAST_PASS), oid(FAILED))]);
}
