//! local_ref: a backup ref back to its local name, only under the four backed-up prefixes. The backup names are
//! written by hand as backup_ref would write them; this file does not run backup_ref.

use super::local_ref;

const PREFIX: &str = "refs/mortar-backup/h1/";

#[test]
fn a_zone_branch_maps_back() {
    assert_eq!(local_ref(PREFIX, "refs/mortar-backup/h1/heads/zone/a"), Some("refs/heads/zone/a".to_string()));
}

#[test]
fn each_backed_up_prefix_maps_back() {
    let pairs = [
        ("refs/mortar-backup/h1/heads/zone/land", "refs/heads/zone/land"),
        ("refs/mortar-backup/h1/heads/req/land/r1", "refs/heads/req/land/r1"),
        ("refs/mortar-backup/h1/mortar/snap/s1", "refs/mortar/snap/s1"),
        ("refs/mortar-backup/h1/mortar/squash/land", "refs/mortar/squash/land"),
    ];
    for (backup, local) in pairs {
        assert_eq!(local_ref(PREFIX, backup), Some(local.to_string()), "{backup}");
    }
}

#[test]
fn main_is_never_a_restore_target() {
    assert_eq!(local_ref(PREFIX, "refs/mortar-backup/h1/heads/main"), None);
}

#[test]
fn refs_outside_the_four_prefixes_map_to_none() {
    let strays = [
        "refs/mortar-backup/h1/heads/integration",
        "refs/mortar-backup/h1/tags/v1",
        "refs/mortar-backup/h1/mortar/oplog",
        "refs/mortar-backup/h1/heads/zones/a",
        "refs/mortar-backup/h1/heads/zone",
    ];
    for backup in strays {
        assert_eq!(local_ref(PREFIX, backup), None, "{backup}");
    }
}

#[test]
fn a_bare_backed_up_prefix_maps_to_none() {
    assert_eq!(local_ref(PREFIX, "refs/mortar-backup/h1/heads/zone/"), None);
    assert_eq!(local_ref(PREFIX, "refs/mortar-backup/h1/mortar/snap/"), None);
}

#[test]
fn another_hosts_backup_maps_to_none() {
    assert_eq!(local_ref(PREFIX, "refs/mortar-backup/h2/heads/zone/a"), None);
    assert_eq!(local_ref(PREFIX, "refs/mortar-backup/h10/heads/zone/a"), None);
}

#[test]
fn a_ref_outside_the_backup_namespace_maps_to_none() {
    assert_eq!(local_ref(PREFIX, "refs/heads/zone/a"), None);
}

/// The inverse rule over many names: each backup name below is written by hand as backup_ref(PREFIX, local)
/// would write it, and local_ref must give the local name back.
#[test]
fn local_ref_maps_many_hand_written_backup_names_back() {
    let pairs = [
        ("refs/heads/zone/a", "refs/mortar-backup/h1/heads/zone/a"),
        ("refs/heads/zone/land", "refs/mortar-backup/h1/heads/zone/land"),
        ("refs/heads/zone/team/land", "refs/mortar-backup/h1/heads/zone/team/land"),
        ("refs/heads/req/a", "refs/mortar-backup/h1/heads/req/a"),
        ("refs/heads/req/land/r1", "refs/mortar-backup/h1/heads/req/land/r1"),
        ("refs/mortar/snap/a", "refs/mortar-backup/h1/mortar/snap/a"),
        ("refs/mortar/snap/land/2026-11-14", "refs/mortar-backup/h1/mortar/snap/land/2026-11-14"),
        ("refs/mortar/squash/land", "refs/mortar-backup/h1/mortar/squash/land"),
        ("refs/mortar/squash/heads/zone/x", "refs/mortar-backup/h1/mortar/squash/heads/zone/x"),
        ("refs/heads/zone/mortar-backup/h1/z", "refs/mortar-backup/h1/heads/zone/mortar-backup/h1/z"),
    ];
    for (local, backup) in pairs {
        assert_eq!(local_ref(PREFIX, backup), Some(local.to_string()), "local_ref {backup}");
    }
}
