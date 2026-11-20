//! Request branch names, and a bad zone or request refused.

use super::request_branch;

#[test]
fn names_req_zone_request() {
    assert_eq!(request_branch("folders", "rf1").unwrap(), "req/folders/rf1");
}

#[test]
fn refuses_a_bad_zone() {
    assert_eq!(request_branch(".z", "r1").unwrap_err().reason, "zone name \".z\" starts with '.' or '-'");
}

#[test]
fn refuses_a_bad_request() {
    assert_eq!(request_branch("folders", "a..b").unwrap_err().reason, "request name \"a..b\" holds \"..\"");
    assert_eq!(request_branch("folders", "").unwrap_err().reason, "request name \"\" is empty");
}
