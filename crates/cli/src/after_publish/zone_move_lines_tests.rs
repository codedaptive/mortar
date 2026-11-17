//! zone_move_lines: what land did with one zone, each ZoneMove arm in the cli's words; a moved zone lists the open
//! requests that rebase at their next join.

use std::path::PathBuf;

use gitio::Oid;
use join::OpenRequest;
use land::ZoneMove;

use super::zone_move_lines;

const OLD: &str = "4b825dc642cb6eb9a060e54bf8d69288fbee4904";
const NEW: &str = "e69de29bb2d1d6434b8b29ae775ad8c2e48c5391";
const COMMIT: &str = "9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e";
const REQUEST_TIP: &str = "3a7d1c9e5b2f8a4d6c0e9b1f7a3d5c8e2b4f6a90";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn request(name: &str) -> OpenRequest {
    OpenRequest { name: name.to_string(), base: oid(OLD), tip: oid(REQUEST_TIP) }
}

#[test]
fn a_moved_zone_with_no_requests_to_rebase_is_one_line() {
    let zone_move = ZoneMove::Moved { zone: "cli".to_string(), old: oid(OLD), new: oid(NEW), rebase: Vec::new() };
    assert_eq!(
        zone_move_lines(&zone_move),
        vec![
            "zone cli moved from 4b825dc642cb6eb9a060e54bf8d69288fbee4904 to e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
                .to_string()
        ]
    );
}

#[test]
fn a_moved_zone_lists_each_request_to_rebase_in_order() {
    let zone_move = ZoneMove::Moved {
        zone: "land".to_string(),
        old: oid(OLD),
        new: oid(NEW),
        rebase: vec![request("r7"), request("fix-bisect")],
    };
    assert_eq!(
        zone_move_lines(&zone_move),
        vec![
            "zone land moved from 4b825dc642cb6eb9a060e54bf8d69288fbee4904 to e69de29bb2d1d6434b8b29ae775ad8c2e48c5391"
                .to_string(),
            "request r7 rebases at its next join".to_string(),
            "request fix-bisect rebases at its next join".to_string(),
        ]
    );
}

#[test]
fn a_current_zone_says_its_branch_did_not_move() {
    let zone_move = ZoneMove::Current { zone: "join".to_string(), tip: oid(NEW) };
    assert_eq!(
        zone_move_lines(&zone_move),
        vec![
            "zone join already holds origin at e69de29bb2d1d6434b8b29ae775ad8c2e48c5391; its branch did not move"
                .to_string()
        ]
    );
}

#[test]
fn a_conflicted_zone_names_the_commit_and_each_file() {
    let zone_move = ZoneMove::Conflicted {
        zone: "cli".to_string(),
        tip: oid(OLD),
        commit: oid(COMMIT),
        files: vec![PathBuf::from("crates/cli/src/lib.rs"), PathBuf::from("Cargo.lock")],
    };
    assert_eq!(
        zone_move_lines(&zone_move),
        vec![
            "zone cli left in place at 4b825dc642cb6eb9a060e54bf8d69288fbee4904: replaying \
             9c1f3e2a7b6d5c4e3f2a1b0c9d8e7f6a5b4c3d2e onto origin conflicts in:"
                .to_string(),
            "crates/cli/src/lib.rs".to_string(),
            "Cargo.lock".to_string(),
        ]
    );
}
