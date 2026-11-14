//! Attempt branch names, and a bad zone, a bad request or attempt 0 refused.

use super::attempt_branch;

#[test]
fn names_try_zone_request_k() {
    assert_eq!(attempt_branch("folders", "WO-folders-6", 1).unwrap(), "try/folders/WO-folders-6/1");
    assert_eq!(attempt_branch("folders", "WO-folders-6", 12).unwrap(), "try/folders/WO-folders-6/12");
}

#[test]
fn names_the_largest_attempt_number() {
    assert_eq!(attempt_branch("z", "r", u32::MAX).unwrap(), "try/z/r/4294967295");
}

#[test]
fn refuses_attempt_zero() {
    assert_eq!(
        attempt_branch("folders", "WO-1", 0).unwrap_err().reason,
        "attempt number 0; attempts are numbered from 1"
    );
}

#[test]
fn refuses_a_bad_zone() {
    assert_eq!(attempt_branch(".z", "WO-1", 1).unwrap_err().reason, "zone name \".z\" starts with '.' or '-'");
}

#[test]
fn refuses_a_bad_request() {
    assert_eq!(attempt_branch("folders", "a..b", 1).unwrap_err().reason, "request name \"a..b\" holds \"..\"");
    assert_eq!(
        attempt_branch("folders", "a/b", 1).unwrap_err().reason,
        "request name \"a/b\" holds a character other than ASCII letters, digits, '-', '_' or '.'"
    );
}
