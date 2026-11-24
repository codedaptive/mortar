//! Landing: a fast-forward landing, a merge landing after origin moved, a conflict that moves nothing, and a
//! second squash of the same zone answered Busy, naming the first, while the first one's repeat answers Landed
//! without touching the op log.

use crate::fixture::{
    refs, state_text, tree, Fixture, ACTOR, ID_A, ID_B, MERGED_FILES, ORIGIN_FILES, SQUASH_FILES,
};
use land::{land, LandAnswer};
use oplog::{oplog_path, read_op_log, ChangeId, OpPhase, RefMove};
use std::path::PathBuf;

#[test]
fn a_squash_on_origin_fast_forwards_integration_to_the_squash() {
    let fx = Fixture::new();
    let squash = fx.squash(&fx.base, &SQUASH_FILES, ID_A);
    let answer = land(&fx.git, &refs(), &squash, ACTOR).unwrap();
    assert_eq!(answer, LandAnswer::Landed { integration: squash.commit.clone() });
    assert_eq!(fx.branch("integration"), squash.commit);
    assert_eq!(fx.branch("main"), fx.base);
    assert_eq!(fx.landing_json(), Some(state_text(ID_A, &squash.commit, &fx.base, &squash.commit)));
    let records = read_op_log(&fx.common).unwrap().records;
    let last = records.last().unwrap();
    assert_eq!((last.kind.as_str(), last.phase), ("land", OpPhase::Done));
    assert_eq!(
        last.moves,
        vec![RefMove {
            refname: "refs/heads/integration".to_string(),
            old: Some(fx.base.clone()),
            new: squash.commit.clone()
        }]
    );
}

#[test]
fn a_squash_behind_a_moved_origin_lands_as_a_merge_commit() {
    let fx = Fixture::new();
    let squash = fx.squash(&fx.base, &SQUASH_FILES, ID_A);
    let squash_before = fx.git.commit_info(&squash.commit).unwrap();
    let origin = fx.move_origin(&ORIGIN_FILES);
    let LandAnswer::Landed { integration: merge } = land(&fx.git, &refs(), &squash, ACTOR).unwrap() else {
        panic!("a clean merge landing must answer Landed");
    };
    let info = fx.git.commit_info(&merge).unwrap();
    assert_eq!(info.parents, vec![origin.clone(), squash.commit.clone()]);
    assert_eq!(info.message, "land klmnopqrstuvwxyzklmnopqrstuvwxyz (land)\n");
    assert_eq!(info.tree, tree(&fx.git, &MERGED_FILES));
    assert_eq!(fx.branch("integration"), merge);
    assert_eq!(fx.landing_json(), Some(state_text(ID_A, &squash.commit, &origin, &merge)));
    assert_eq!(fx.git.commit_info(&squash.commit).unwrap(), squash_before);
}

#[test]
fn a_conflicting_squash_moves_nothing_and_writes_no_state() {
    let fx = Fixture::new();
    let squash = fx.squash(&fx.base, &[("a.txt", "squash\n")], ID_A);
    let origin = fx.move_origin(&[("a.txt", "origin\n")]);
    assert_eq!(
        land(&fx.git, &refs(), &squash, ACTOR).unwrap(),
        LandAnswer::Conflicted { files: vec![PathBuf::from("a.txt")] }
    );
    assert_eq!(fx.branch("integration"), origin);
    assert_eq!(fx.landing_json(), None);
    assert_eq!(read_op_log(&fx.common).unwrap().records, vec![]);
}

#[test]
fn a_second_squash_of_the_same_zone_is_busy_and_the_first_ones_repeat_is_landed_with_the_log_unchanged() {
    let fx = Fixture::new();
    let first = fx.squash(&fx.base, &SQUASH_FILES, ID_A);
    let second = fx.squash(&fx.base, &[("a.txt", "base\n"), ("b.txt", "second\n")], ID_B);
    let answer = land(&fx.git, &refs(), &first, ACTOR).unwrap();
    assert_eq!(answer, LandAnswer::Landed { integration: first.commit.clone() });
    let log = std::fs::read(oplog_path(&fx.common)).unwrap();
    assert_eq!(
        land(&fx.git, &refs(), &second, ACTOR).unwrap(),
        LandAnswer::Busy { change_id: ChangeId::parse("klmnopqrstuvwxyzklmnopqrstuvwxyz").unwrap() }
    );
    assert_eq!(fx.branch("integration"), first.commit);
    let repeat = land(&fx.git, &refs(), &first, ACTOR).unwrap();
    assert_eq!(repeat, LandAnswer::Landed { integration: first.commit.clone() });
    assert_eq!(fx.branch("integration"), first.commit);
    assert_eq!(std::fs::read(oplog_path(&fx.common)).unwrap(), log);
    assert_eq!(fx.landing_json(), Some(state_text(ID_A, &first.commit, &fx.base, &first.commit)));
}
