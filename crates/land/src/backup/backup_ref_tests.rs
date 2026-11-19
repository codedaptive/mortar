//! backup_ref: a local ref's path under the host's backup prefix.

use super::backup_ref;

#[test]
fn a_zone_branch_maps_under_the_prefix() {
    assert_eq!(backup_ref("refs/mortar-backup/h1/", "refs/heads/zone/a"), "refs/mortar-backup/h1/heads/zone/a");
}

#[test]
fn a_request_branch_keeps_its_nested_path() {
    assert_eq!(
        backup_ref("refs/mortar-backup/h1/", "refs/heads/req/land/r1"),
        "refs/mortar-backup/h1/heads/req/land/r1"
    );
}

#[test]
fn a_snapshot_ref_maps_under_the_prefix() {
    assert_eq!(backup_ref("refs/mortar-backup/h2/", "refs/mortar/snap/x"), "refs/mortar-backup/h2/mortar/snap/x");
}

#[test]
fn a_squash_ref_maps_under_the_prefix() {
    assert_eq!(
        backup_ref("refs/mortar-backup/h1/", "refs/mortar/squash/land"),
        "refs/mortar-backup/h1/mortar/squash/land"
    );
}
