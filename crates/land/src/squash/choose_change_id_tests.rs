//! choose_change_id: keep the prior squash's ID only when it is not in origin and has one.

use super::{choose_change_id, PriorSquash};
use gitio::{CommitInfo, Oid, Person};
use oplog::ChangeId;

/// The one helper building values only a constructor makes (Oid::parse, ChangeId::parse).
fn prior(with_id: bool, in_origin: bool) -> PriorSquash {
    let oid = |hex: &str| Oid::parse(hex).unwrap();
    PriorSquash {
        commit: oid("1111111111111111111111111111111111111111"),
        info: CommitInfo {
            tree: oid("2222222222222222222222222222222222222222"),
            parents: vec![oid("3333333333333333333333333333333333333333")],
            author: Person { name: "A".to_string(), email: "a@x".to_string(), date: None },
            message: "land: 1 requests\n".to_string(),
        },
        change_id: with_id.then(|| ChangeId::parse("zzzzyyyyxxxxwwwwvvvvuuuuttttssss").unwrap()),
        in_origin,
    }
}

#[test]
fn no_prior_draws_a_new_id() {
    assert_eq!(choose_change_id(None), None);
}

#[test]
fn a_prior_already_in_origin_draws_a_new_id() {
    assert_eq!(choose_change_id(Some(&prior(true, true))), None);
}

#[test]
fn a_prior_not_in_origin_with_an_id_keeps_it() {
    let prior = prior(true, false);
    assert_eq!(choose_change_id(Some(&prior)), prior.change_id);
}

#[test]
fn a_prior_not_in_origin_without_an_id_draws_a_new_id() {
    assert_eq!(choose_change_id(Some(&prior(false, false))), None);
}
