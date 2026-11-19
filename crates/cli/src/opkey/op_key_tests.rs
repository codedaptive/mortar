//! op_key: --key wins; else a fixed key for fixed inputs, the same on a repeat, and different when any one input
//! changes.
//! Pinned keys: 64-bit FNV-1a of the NUL-ended fields, computed by a separate implementation, not by op_key or hash_key.

use std::path::Path;

use gitio::Oid;
use oplog::RefMove;

use super::op_key;

fn oid(c: char) -> Oid {
    Oid::parse(&c.to_string().repeat(40)).expect("a valid oid")
}

fn zone_move(old: Option<char>, new: char) -> RefMove {
    RefMove { refname: "refs/heads/zone/a".to_string(), old: old.map(oid), new: oid(new) }
}

fn key(verb: &str, folder: &str, moves: &[RefMove]) -> String {
    op_key(None, verb, Path::new(folder), moves)
}

#[test]
fn fixed_inputs_give_a_fixed_key() {
    assert_eq!(key("join", "/r/a/r1", &[zone_move(Some('a'), 'b')]), "join-b837b2f1dcd1cb7d");
}

#[test]
fn an_absent_old_value_hashes_as_none() {
    assert_eq!(key("join", "/r/a/r1", &[zone_move(None, 'b')]), "join-ac64ac52cb189b8b");
}

#[test]
fn no_moves_hash_the_verb_and_folder() {
    assert_eq!(key("init", "/r", &[]), "init-9ced48734e320c20");
}

#[test]
fn a_repeat_gives_the_same_key() {
    let moves = [zone_move(Some('a'), 'b'), zone_move(None, 'c')];
    assert_eq!(key("land", "/r/x", &moves), key("land", "/r/x", &moves));
}

#[test]
fn changing_any_one_input_changes_the_key() {
    let base = key("join", "/r/a/r1", &[zone_move(Some('a'), 'b')]);
    let other_ref = RefMove { refname: "refs/heads/zone/b".to_string(), old: Some(oid('a')), new: oid('b') };
    let changed = [
        key("land", "/r/a/r1", &[zone_move(Some('a'), 'b')]),
        key("join", "/r/a/r2", &[zone_move(Some('a'), 'b')]),
        key("join", "/r/a/r1", &[other_ref]),
        key("join", "/r/a/r1", &[zone_move(Some('c'), 'b')]),
        key("join", "/r/a/r1", &[zone_move(None, 'b')]),
        key("join", "/r/a/r1", &[zone_move(Some('a'), 'c')]),
        key("join", "/r/a/r1", &[]),
    ];
    for other in changed {
        assert_ne!(other, base);
    }
}

#[test]
fn the_flag_wins() {
    assert_eq!(op_key(Some("harness-7"), "join", Path::new("/r"), &[zone_move(None, 'b')]), "harness-7");
}
