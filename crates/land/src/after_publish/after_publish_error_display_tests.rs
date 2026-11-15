//! Display for AfterPublishError's NoSquashOf refusal as literal text naming the zone and saying it was not moved.
//! The Unrelated and SquashOfNotInZone arms print gitio's Oid Display and are Nagatha RITs (owed, S-101); the Git,
//! Squash, Join and Land arms embed another error's text and have no case here.

use super::AfterPublishError;

#[test]
fn no_squash_of_names_the_zone_and_says_it_was_not_moved() {
    let error = AfterPublishError::NoSquashOf { zone: "land".to_string() };
    assert_eq!(
        error.to_string(),
        "no squash of zone land on origin carries a Mortar-Squash-Of trailer; the zone was not moved"
    );
}
