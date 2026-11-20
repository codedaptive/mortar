//! squash_of_in: the squash-of of this zone's squash; None for another zone's, for no Mortar-Zone and for no
//! Mortar-Squash-Of. Keys match in any ASCII case (lowercase, mixed); when Mortar-Zone or Mortar-Squash-Of
//! repeats, the last pair counts. Keys are written as literal trailer text, as a squash commit carries them.

use super::squash_of_in;

const SQUASH_OF: &str = "4444444444444444444444444444444444444444";

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter().map(|(key, value)| (key.to_string(), value.to_string())).collect()
}

#[test]
fn this_zones_squash_names_its_squash_of() {
    let trailers = pairs(&[
        ("Mortar-Change-Id", "klmnopqrstuvwxyzklmnopqrstuvwxyz"),
        ("Mortar-Zone", "land"),
        ("Mortar-Squash-Of", SQUASH_OF),
    ]);
    assert_eq!(squash_of_in(&trailers, "land"), Some(SQUASH_OF));
}

#[test]
fn another_zones_squash_is_none() {
    let trailers = pairs(&[("Mortar-Zone", "join"), ("Mortar-Squash-Of", SQUASH_OF)]);
    assert_eq!(squash_of_in(&trailers, "land"), None);
}

#[test]
fn a_squash_of_with_no_zone_trailer_is_none() {
    let trailers = pairs(&[("Mortar-Squash-Of", SQUASH_OF)]);
    assert_eq!(squash_of_in(&trailers, "land"), None);
}

#[test]
fn this_zones_commit_with_no_squash_of_is_none() {
    let trailers = pairs(&[("Mortar-Zone", "land"), ("Mortar-Request", "r1")]);
    assert_eq!(squash_of_in(&trailers, "land"), None);
}

#[test]
fn no_trailers_is_none() {
    assert_eq!(squash_of_in(&[], "land"), None);
}

#[test]
fn a_zone_name_that_only_starts_the_same_is_another_zone() {
    let trailers = pairs(&[("Mortar-Zone", "landing"), ("Mortar-Squash-Of", SQUASH_OF)]);
    assert_eq!(squash_of_in(&trailers, "land"), None);
}

#[test]
fn lowercase_keys_are_found() {
    let trailers = pairs(&[("mortar-zone", "land"), ("mortar-squash-of", SQUASH_OF)]);
    assert_eq!(squash_of_in(&trailers, "land"), Some(SQUASH_OF));
}

#[test]
fn mixed_case_keys_are_found() {
    let trailers = pairs(&[("MORTAR-ZONE", "land"), ("mortar-Squash-Of", SQUASH_OF)]);
    assert_eq!(squash_of_in(&trailers, "land"), Some(SQUASH_OF));
}

#[test]
fn a_later_zone_of_another_zone_wins_and_is_none() {
    let trailers = pairs(&[
        ("Mortar-Zone", "land"),
        ("Mortar-Squash-Of", SQUASH_OF),
        ("Mortar-Zone", "join"),
    ]);
    assert_eq!(squash_of_in(&trailers, "land"), None);
}

#[test]
fn a_later_zone_of_this_zone_wins_and_names_the_squash_of() {
    let trailers = pairs(&[
        ("Mortar-Zone", "join"),
        ("Mortar-Zone", "land"),
        ("Mortar-Squash-Of", SQUASH_OF),
    ]);
    assert_eq!(squash_of_in(&trailers, "land"), Some(SQUASH_OF));
}

#[test]
fn a_repeated_squash_of_gives_the_last_value() {
    let trailers = pairs(&[
        ("Mortar-Zone", "land"),
        ("Mortar-Squash-Of", "1111111111111111111111111111111111111111"),
        ("Mortar-Squash-Of", SQUASH_OF),
    ]);
    assert_eq!(squash_of_in(&trailers, "land"), Some(SQUASH_OF));
}
