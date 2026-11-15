//! replay_record_ref: the zone and the source's hex under refs/mortar/replayed/.

use super::replay_record_ref;

#[test]
fn the_ref_names_the_zone_then_the_source() {
    assert_eq!(
        replay_record_ref("land", "1111111111111111111111111111111111111111"),
        "refs/mortar/replayed/land/1111111111111111111111111111111111111111"
    );
}
