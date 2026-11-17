//! default_zone_start (S-149): a bare `mr folder zone` on a missing zone branch starts it at the origin branch tip;
//! a given --start, or a zone branch that exists, gives no default.

use super::default_zone_start;

#[test]
fn no_start_and_no_zone_branch_starts_at_the_origin_branch() {
    assert_eq!(default_zone_start(None, false, "main"), Some("refs/heads/main".to_string()));
}

#[test]
fn the_origin_branch_name_comes_from_the_argument() {
    assert_eq!(default_zone_start(None, false, "trunk/next"), Some("refs/heads/trunk/next".to_string()));
}

#[test]
fn an_existing_zone_branch_gets_no_default() {
    assert_eq!(default_zone_start(None, true, "main"), None);
}

#[test]
fn a_given_start_gets_no_default_when_the_zone_branch_is_missing() {
    assert_eq!(default_zone_start(Some("abc123"), false, "main"), None);
}

#[test]
fn a_given_start_gets_no_default_when_the_zone_branch_exists() {
    assert_eq!(default_zone_start(Some("abc123"), true, "main"), None);
}
