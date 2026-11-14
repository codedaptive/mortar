//! Snapshot ref names, and a bad zone or request refused.

use super::snap_ref;

#[test]
fn names_refs_mortar_snap_zone_request() {
    assert_eq!(snap_ref("folders", "WO-folders-5").unwrap(), "refs/mortar/snap/folders/WO-folders-5");
}

#[test]
fn refuses_a_bad_zone() {
    assert_eq!(snap_ref(".z", "WO-1").unwrap_err().reason, "zone name \".z\" starts with '.' or '-'");
}

#[test]
fn refuses_a_bad_request() {
    assert_eq!(snap_ref("folders", "a..b").unwrap_err().reason, "request name \"a..b\" holds \"..\"");
}
