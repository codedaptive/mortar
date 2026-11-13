//! check_ref_moves: usable names pass; the first empty or NUL-holding name is refused by name.

use gitio::Oid;

use super::check_ref_moves;
use crate::store::{OplogError, RefMove};

fn mv(refname: &str, old: Option<&str>, new: &str) -> RefMove {
    RefMove {
        refname: refname.to_string(),
        old: old.map(|h| Oid::parse(h).unwrap()),
        new: Oid::parse(new).unwrap(),
    }
}

fn bad_move(result: Result<(), OplogError>) -> (String, String) {
    match result {
        Err(OplogError::BadMove { refname, reason }) => (refname, reason),
        other => panic!("expected BadMove, got {other:?}"),
    }
}

#[test]
fn usable_moves_pass_including_a_delete() {
    let moves = vec![
        mv(
            "refs/heads/landing",
            Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        ),
        mv("refs/heads/new-branch", None, "cccccccccccccccccccccccccccccccccccccccc"),
        mv(
            "refs/heads/gone",
            Some("dddddddddddddddddddddddddddddddddddddddd"),
            "0000000000000000000000000000000000000000",
        ),
    ];
    assert!(check_ref_moves(&moves).is_ok());
}

#[test]
fn no_moves_pass() {
    assert!(check_ref_moves(&[]).is_ok());
}

#[test]
fn an_empty_name_is_refused() {
    let moves = vec![mv("", None, "cccccccccccccccccccccccccccccccccccccccc")];
    assert_eq!(
        bad_move(check_ref_moves(&moves)),
        ("".to_string(), "is empty".to_string())
    );
}

#[test]
fn a_name_holding_a_nul_is_refused() {
    let moves = vec![mv("refs/heads/a\0b", None, "cccccccccccccccccccccccccccccccccccccccc")];
    assert_eq!(
        bad_move(check_ref_moves(&moves)),
        ("refs/heads/a\0b".to_string(), "holds a NUL".to_string())
    );
}

#[test]
fn the_first_bad_move_is_the_one_named() {
    let moves = vec![
        mv("refs/heads/ok", None, "cccccccccccccccccccccccccccccccccccccccc"),
        mv("refs/heads/x\0y", None, "cccccccccccccccccccccccccccccccccccccccc"),
        mv("", None, "cccccccccccccccccccccccccccccccccccccccc"),
    ];
    assert_eq!(
        bad_move(check_ref_moves(&moves)),
        ("refs/heads/x\0y".to_string(), "holds a NUL".to_string())
    );
}

#[test]
fn the_refusal_reads_with_the_ref_and_the_fix() {
    let moves = vec![mv("", None, "cccccccccccccccccccccccccccccccccccccccc")];
    assert_eq!(
        check_ref_moves(&moves).unwrap_err().to_string(),
        "op cannot be applied: ref \"\" is empty; fix the op's moves and run again"
    );
}
