//! Cases 1, 2, 4, 5 and 6: publish moves the remote and then the local origin; a remote moved by someone else
//! answers OriginMoved with integration back on the fetched origin and nothing lost, and the same squash lands
//! again; with no landing state there is nothing to publish and the remote is untouched; a push the remote's hook
//! refuses with its main still at expected is PushRefused with the landing still in flight; a push refused after
//! the remote's main moved answers OriginMoved.

use crate::fixture::{refs, state_text, Fixture, ACTOR, ID_A, SQUASH_REF};
use gitio::PushRefusal;
use land::{land, publish, LandAnswer, PublishAnswer, PublishError};
use oplog::{read_op_log, OpPhase, RefMove};

#[test]
fn publish_moves_the_remote_then_the_local_origin_and_cleans_up() {
    let (fx, squash) = Fixture::landed();
    assert_eq!(publish(&fx.git, &refs(), ACTOR).unwrap(), PublishAnswer::Published { origin: squash.commit.clone() });
    assert_eq!(fx.remote_main(), Some(squash.commit.clone()));
    assert_eq!(fx.branch("main"), squash.commit);
    assert_eq!(fx.branch("integration"), squash.commit);
    assert_eq!(fx.landing_json(), None);
    assert_eq!(fx.git.read_ref(SQUASH_REF).unwrap(), None);
    let records = read_op_log(&fx.common).unwrap().records;
    let phases: Vec<(&str, OpPhase)> = records.iter().map(|record| (record.kind.as_str(), record.phase)).collect();
    assert_eq!(
        phases,
        vec![
            ("land", OpPhase::Intent),
            ("land", OpPhase::Done),
            ("publish-push", OpPhase::Intent),
            ("publish-push", OpPhase::Done),
            ("publish-origin", OpPhase::Intent),
            ("publish-origin", OpPhase::Done),
            ("publish-squash", OpPhase::Intent),
            ("publish-squash", OpPhase::Done),
        ]
    );
    let push_move =
        vec![RefMove { refname: "refs/heads/main".to_string(), old: Some(fx.base.clone()), new: squash.commit.clone() }];
    assert_eq!(records[2].moves, push_move);
    assert_eq!(records[3].moves, push_move);
}

#[test]
fn a_remote_moved_by_someone_else_is_fetched_backed_out_onto_and_lands_again() {
    let (fx, squash) = Fixture::landed();
    let moved = fx.someone_else_pushes("main");
    assert_eq!(publish(&fx.git, &refs(), ACTOR).unwrap(), PublishAnswer::OriginMoved { found: moved.clone() });
    assert_eq!(fx.branch("main"), moved);
    assert_eq!(fx.branch("integration"), moved);
    assert_eq!(fx.landing_json(), None);
    assert_eq!(fx.git.commit_info(&squash.commit).unwrap().parents, vec![fx.base.clone()]);
    assert_eq!(fx.git.read_ref(SQUASH_REF).unwrap(), Some(squash.commit.clone()));
    assert_eq!(fx.remote_main(), Some(moved.clone()));
    let LandAnswer::Landed { integration: merge } = land(&fx.git, &refs(), &squash, ACTOR).unwrap() else {
        panic!("the same squash must land again on the fetched origin");
    };
    assert_eq!(fx.git.commit_info(&merge).unwrap().parents, vec![moved.clone(), squash.commit.clone()]);
    assert_eq!(fx.branch("integration"), merge);
}

#[test]
fn with_no_landing_state_there_is_nothing_to_publish_and_the_remote_is_untouched() {
    let fx = Fixture::new();
    assert_eq!(publish(&fx.git, &refs(), ACTOR).unwrap(), PublishAnswer::NothingToPublish);
    assert_eq!(fx.remote_main(), Some(fx.base.clone()));
    assert_eq!(fx.branch("main"), fx.base);
}

#[test]
fn a_push_refused_with_the_remote_still_at_expected_is_push_refused_and_keeps_the_landing() {
    let (fx, squash) = Fixture::landed();
    fx.install_pre_receive("#!/bin/sh\nexit 1\n");
    let result = publish(&fx.git, &refs(), ACTOR);
    assert!(
        matches!(
            &result,
            Err(PublishError::PushRefused { refname, refusal: PushRefusal::Hook }) if refname == "refs/heads/main"
        ),
        "{result:?}"
    );
    assert_eq!(fx.remote_main(), Some(fx.base.clone()));
    assert_eq!(fx.branch("main"), fx.base);
    assert_eq!(fx.branch("integration"), squash.commit);
    assert_eq!(fx.landing_json(), Some(state_text(ID_A, &squash.commit, &fx.base, &squash.commit)));
}

#[test]
fn a_remote_moved_while_the_push_was_refused_answers_origin_moved() {
    let (fx, squash) = Fixture::landed();
    let moved = fx.someone_else_pushes("other");
    // The hook runs inside the push's object quarantine, where git refuses ref updates; X is already in the
    // remote's own objects, so the hook leaves the quarantine to move main to X, then declines the push.
    fx.install_pre_receive(&format!(
        "#!/bin/sh\nset -e\nunset GIT_QUARANTINE_PATH GIT_OBJECT_DIRECTORY GIT_ALTERNATE_OBJECT_DIRECTORIES\n\
         git update-ref refs/heads/main {moved} {}\nexit 1\n",
        fx.base
    ));
    assert_eq!(publish(&fx.git, &refs(), ACTOR).unwrap(), PublishAnswer::OriginMoved { found: moved.clone() });
    assert_eq!(fx.remote_main(), Some(moved.clone()));
    assert_eq!(fx.branch("main"), moved);
    assert_eq!(fx.branch("integration"), moved);
    assert_eq!(fx.landing_json(), None);
    assert_eq!(fx.git.read_ref(SQUASH_REF).unwrap(), Some(squash.commit.clone()));
}
