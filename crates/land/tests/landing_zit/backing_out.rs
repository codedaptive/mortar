//! Backing out: back_out after a merge landing restores integration, and a second back_out has nothing to back out.

use crate::fixture::{refs, Fixture, ACTOR, ID_A, ORIGIN_FILES, SQUASH_FILES};
use land::{back_out, land, BackOutAnswer, LandAnswer};
use oplog::{read_op_log, ChangeId, OpPhase, RefMove};

#[test]
fn back_out_restores_integration_then_has_nothing_to_back_out() {
    let fx = Fixture::new();
    let squash = fx.squash(&fx.base, &SQUASH_FILES, ID_A);
    let origin = fx.move_origin(&ORIGIN_FILES);
    let LandAnswer::Landed { integration: merge } = land(&fx.git, &refs(), &squash, ACTOR).unwrap() else {
        panic!("a clean merge landing must answer Landed");
    };
    assert_eq!(
        back_out(&fx.git, &refs(), ACTOR).unwrap(),
        BackOutAnswer::BackedOut { change_id: ChangeId::parse("klmnopqrstuvwxyzklmnopqrstuvwxyz").unwrap() }
    );
    assert_eq!(fx.branch("integration"), origin);
    assert_eq!(fx.branch("main"), origin);
    assert_eq!(fx.landing_json(), None);
    assert_eq!(fx.git.commit_info(&squash.commit).unwrap().parents, vec![fx.base.clone()]);
    let records = read_op_log(&fx.common).unwrap().records;
    let last = records.last().unwrap();
    assert_eq!((last.kind.as_str(), last.phase), ("land-back-out", OpPhase::Done));
    assert_eq!(
        last.moves,
        vec![RefMove { refname: "refs/heads/integration".to_string(), old: Some(merge), new: origin.clone() }]
    );
    assert_eq!(back_out(&fx.git, &refs(), ACTOR).unwrap(), BackOutAnswer::NothingToBackOut);
    assert_eq!(fx.branch("integration"), origin);
}
