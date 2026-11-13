//! ref_updates: each move becomes one update, in order, as given.

use gitio::{Oid, RefUpdate};

use super::ref_updates;
use crate::store::RefMove;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn moves_become_updates_in_order_as_given() {
    let moves = vec![
        RefMove {
            refname: "refs/heads/landing".to_string(),
            old: Some(oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),
            new: oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
        },
        RefMove {
            refname: "refs/heads/new-branch".to_string(),
            old: None,
            new: oid("cccccccccccccccccccccccccccccccccccccccc"),
        },
        RefMove {
            refname: "refs/heads/gone".to_string(),
            old: Some(oid("dddddddddddddddddddddddddddddddddddddddd")),
            new: oid("0000000000000000000000000000000000000000"),
        },
    ];
    assert_eq!(
        ref_updates(&moves),
        vec![
            RefUpdate {
                refname: "refs/heads/landing".to_string(),
                new: oid("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
                old: Some(oid("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")),
            },
            RefUpdate {
                refname: "refs/heads/new-branch".to_string(),
                new: oid("cccccccccccccccccccccccccccccccccccccccc"),
                old: None,
            },
            RefUpdate {
                refname: "refs/heads/gone".to_string(),
                new: oid("0000000000000000000000000000000000000000"),
                old: Some(oid("dddddddddddddddddddddddddddddddddddddddd")),
            },
        ]
    );
}

#[test]
fn no_moves_give_no_updates() {
    assert_eq!(ref_updates(&[]), Vec::<RefUpdate>::new());
}
