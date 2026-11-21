//! branchless_request_line: the request and the zone appear as given, in the cli's one sentence.

use super::branchless_request_line;

#[test]
fn a_request_with_a_space_is_named_with_its_zone() {
    assert_eq!(
        branchless_request_line("has space", "core"),
        "request has space in zone core is not a name a branch can carry"
    );
}

#[test]
fn the_request_comes_first_and_the_zone_second() {
    assert_eq!(
        branchless_request_line("a..b", "cli"),
        "request a..b in zone cli is not a name a branch can carry"
    );
}
