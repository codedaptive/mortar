//! land::withdraw: deletes the zone's squash ref in one land-withdraw op and answers the commit and change ID it
//! held; with no squash ref it answers Absent and writes nothing; a squash already in origin or the one the landing
//! state names is refused with nothing moved and no op; a squash after a withdraw draws a new change ID.

use crate::fixture::{made, refs, Fixture, ACTOR, SQUASH_REF, ZONE};
use land::{land, squash, withdraw, LandAnswer, WithdrawAnswer, WithdrawError};

#[test]
fn a_withdraw_deletes_the_squash_ref_and_answers_its_commit_and_change_id() {
    let fx = Fixture::new();
    let squashed = fx.squashed();
    let refs_before = fx.ref_lines();
    let squash_line = format!("{SQUASH_REF} {}", squashed.commit);
    assert!(refs_before.contains(&squash_line));
    let log_before = fx.oplog().unwrap();

    let answer = withdraw(&fx.land_git(), &refs(), ZONE, ACTOR).unwrap();

    let expected = WithdrawAnswer::Withdrawn { commit: squashed.commit.clone(), change_id: Some(squashed.change_id) };
    assert_eq!(answer, expected);
    assert_eq!(fx.ref_of(SQUASH_REF), None);
    let others: Vec<String> = refs_before.into_iter().filter(|line| *line != squash_line).collect();
    assert_eq!(fx.ref_lines(), others);
    let log_after = fx.oplog().unwrap();
    assert!(log_after.len() > log_before.len() && log_after.starts_with(&log_before));
    let text = String::from_utf8(log_after).unwrap();
    assert!(text.lines().last().unwrap().contains(r#""kind":"land-withdraw""#));
}

#[test]
fn a_repeated_withdraw_is_absent_and_writes_nothing() {
    let fx = Fixture::new();
    fx.squashed();
    withdraw(&fx.land_git(), &refs(), ZONE, ACTOR).unwrap();
    let refs_before = fx.ref_lines();
    let log_before = fx.oplog();

    let again = withdraw(&fx.land_git(), &refs(), ZONE, ACTOR).unwrap();

    assert_eq!(again, WithdrawAnswer::Absent);
    assert_eq!(fx.ref_lines(), refs_before);
    assert!(log_before.is_some());
    assert_eq!(fx.oplog(), log_before);
}

#[test]
fn a_published_squash_is_refused_and_moves_nothing() {
    let fx = Fixture::new();
    let squashed = fx.squashed();
    let main_tip = fx.commit_on(&squashed.commit.to_string(), &[("p", "past\n")], "past the squash", "r2");
    fx.set_branch("main", &main_tip);
    let refs_before = fx.ref_lines();
    let log_before = fx.oplog();

    match withdraw(&fx.land_git(), &refs(), ZONE, ACTOR) {
        Err(WithdrawError::Published { zone, commit, origin }) => {
            assert_eq!(zone, "land");
            assert_eq!(commit, squashed.commit);
            assert_eq!(origin.to_string(), main_tip);
        }
        other => panic!("expected Published, got {other:?}"),
    }
    assert_eq!(fx.ref_of(SQUASH_REF), Some(squashed.commit.to_string()));
    assert_eq!(fx.ref_lines(), refs_before);
    assert_eq!(fx.oplog(), log_before);
}

#[test]
fn the_squash_in_the_landing_state_is_refused_and_moves_nothing() {
    let fx = Fixture::new();
    let squashed = fx.squashed();
    let landed = land(&fx.land_git(), &refs(), &squashed, ACTOR).unwrap();
    assert_eq!(landed, LandAnswer::Landed { integration: squashed.commit.clone() });
    let refs_before = fx.ref_lines();
    let log_before = fx.oplog();

    match withdraw(&fx.land_git(), &refs(), ZONE, ACTOR) {
        Err(WithdrawError::Landing { zone, commit, change_id }) => {
            assert_eq!(zone, "land");
            assert_eq!(commit, squashed.commit);
            assert_eq!(change_id, squashed.change_id);
        }
        other => panic!("expected Landing, got {other:?}"),
    }
    assert_eq!(fx.ref_of(SQUASH_REF), Some(squashed.commit.to_string()));
    assert_eq!(fx.ref_of("refs/heads/integration"), Some(squashed.commit.to_string()));
    assert_eq!(fx.ref_lines(), refs_before);
    assert_eq!(fx.oplog(), log_before);
}

#[test]
fn squash_withdraw_squash_draws_a_new_change_id() {
    let fx = Fixture::new();
    let first = fx.squashed();
    withdraw(&fx.land_git(), &refs(), ZONE, ACTOR).unwrap();

    let second = made(squash(&fx.land_git(), &refs(), ZONE, ACTOR).unwrap());

    assert_ne!(second.change_id, first.change_id);
    assert_eq!(fx.ref_of(SQUASH_REF), Some(second.commit.to_string()));
}
