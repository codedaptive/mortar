//! Zone branch names, and a bad zone refused.

use super::zone_branch;

#[test]
fn names_zone_slash_zone() {
    assert_eq!(zone_branch("folders").unwrap(), "zone/folders");
    assert_eq!(zone_branch("a_b.c-9").unwrap(), "zone/a_b.c-9");
}

#[test]
fn refuses_a_bad_zone() {
    assert_eq!(zone_branch("a.").unwrap_err().reason, "zone name \"a.\" ends with \".lock\" or '.'");
    assert_eq!(zone_branch("").unwrap_err().reason, "zone name \"\" is empty");
}
