//! squash_ref: the zone's name under refs/mortar/squash/ (S-57).

use super::squash_ref;

#[test]
fn names_the_zone_under_refs_mortar_squash() {
    assert_eq!(squash_ref("land"), "refs/mortar/squash/land");
}

#[test]
fn keeps_a_zone_name_with_a_slash_whole() {
    assert_eq!(squash_ref("team/land"), "refs/mortar/squash/team/land");
}
