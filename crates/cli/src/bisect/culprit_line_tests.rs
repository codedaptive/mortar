//! culprit_line: what the culprit's squash trailers name, for each of the four ways they can be present or absent.

use gitio::Oid;
use land::Culprit;
use oplog::ChangeId;

use super::culprit_line;

const COMMIT: &str = "9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e";
const CHANGE: &str = "zqxnmwkrtpvsloyuytlmnwqxrkopzsvu";

fn culprit(change_id: Option<&str>, zone: Option<&str>) -> Culprit {
    Culprit {
        commit: Oid::parse(COMMIT).unwrap(),
        change_id: change_id.map(|id| ChangeId::parse(id).unwrap()),
        zone: zone.map(ToString::to_string),
    }
}

#[test]
fn change_id_and_zone_are_both_named() {
    assert_eq!(
        culprit_line(&culprit(Some(CHANGE), Some("cli"))),
        "its squash is change zqxnmwkrtpvsloyuytlmnwqxrkopzsvu from zone cli"
    );
}

#[test]
fn a_change_id_without_a_zone_says_the_zone_trailer_is_missing() {
    assert_eq!(
        culprit_line(&culprit(Some(CHANGE), None)),
        "its squash is change zqxnmwkrtpvsloyuytlmnwqxrkopzsvu; it carries no Mortar-Zone trailer"
    );
}

#[test]
fn a_zone_without_a_change_id_says_the_change_id_trailer_is_missing() {
    assert_eq!(
        culprit_line(&culprit(None, Some("land"))),
        "its squash is from zone land; it carries no Mortar-Change-Id trailer"
    );
}

#[test]
fn neither_trailer_says_nothing_is_named() {
    assert_eq!(culprit_line(&culprit(None, None)), "it carries no squash trailers, so no change ID or zone is named");
}
