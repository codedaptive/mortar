//! Cases 1, 2 and 4 to 10: publish moves the remote and then the local origin, its push intent and done record
//! marked with the remote `origin`; a remote moved by someone else answers OriginMoved with integration back on the
//! fetched origin and nothing lost, and the same squash lands again; with no landing state there is nothing to
//! publish and the remote is untouched; a push the remote's hook refuses with its main still at expected is
//! PushRefused with the landing still in flight; a push refused after the remote's main moved answers OriginMoved,
//! its push intent closed failed naming what the remote holds. Cases 7 and 8 are the S-103 repeat: a push intent
//! left open (opened by the fixture after a real land) is closed done when the remote is already at the squash, and
//! closed failed and pushed again under a second intent when it is still at the base. Case 9: with the remote past
//! the squash (someone else pushed on top of it), the open intent is closed done and nothing is pushed again.
//! Case 10: a remote already past the squash with no push intent in the log is published with no push and no
//! publish-push record, and nothing is backed out (S-110).

use crate::fixture::{refs, state_text, Fixture, ACTOR, ID_A, OPEN_PUSH_OP_ID, PAST_SQUASH_FILES, SQUASH_REF};
use gitio::PushRefusal;
use land::{land, publish, LandAnswer, PublishAnswer, PublishError};
use oplog::{open_intents, read_op_log, recover, OpPhase, OpResult, RefMove};

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
    let push_move = vec![RefMove {
        refname: "refs/heads/main".to_string(),
        old: Some(fx.base.clone()),
        new: squash.commit.clone(),
    }];
    assert_eq!(records[2].moves, push_move);
    assert_eq!(records[3].moves, push_move);
    let origin = Some("origin".to_string());
    assert_eq!(
        fx.push_log(),
        vec![(OpPhase::Intent, None, origin.clone()), (OpPhase::Done, Some(OpResult::Moved), origin)]
    );
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
    let origin = Some("origin".to_string());
    let not_moved = OpResult::NotMoved { refs: vec![("refs/heads/main".to_string(), Some(moved.clone()))] };
    assert_eq!(
        fx.push_log(),
        vec![(OpPhase::Intent, None, origin.clone()), (OpPhase::Failed, Some(not_moved), origin)]
    );
}

#[test]
fn an_open_push_intent_after_the_push_is_closed_done_on_the_repeat() {
    let (fx, squash) = Fixture::landed();
    fx.open_push_intent(&squash.commit);
    fx.push_remote_main(&squash.commit);
    assert_open_after_recover(&fx);
    assert_eq!(publish(&fx.git, &refs(), ACTOR).unwrap(), PublishAnswer::Published { origin: squash.commit.clone() });
    let origin = Some("origin".to_string());
    assert_eq!(
        push_ops(&fx),
        vec![
            (OPEN_PUSH_OP_ID.to_string(), OpPhase::Intent, None, origin.clone()),
            (OPEN_PUSH_OP_ID.to_string(), OpPhase::Done, Some(OpResult::Moved), origin),
        ]
    );
    assert_eq!(fx.remote_main(), Some(squash.commit.clone()));
    assert_eq!(fx.branch("main"), squash.commit);
}

#[test]
fn an_open_push_intent_before_the_push_is_closed_failed_and_pushed_again() {
    let (fx, squash) = Fixture::landed();
    fx.open_push_intent(&squash.commit);
    assert_open_after_recover(&fx);
    assert_eq!(fx.remote_main(), Some(fx.base.clone()));
    assert_eq!(publish(&fx.git, &refs(), ACTOR).unwrap(), PublishAnswer::Published { origin: squash.commit.clone() });
    let records = fx.push_records();
    assert_eq!(records.len(), 4, "{records:?}");
    let second = records[2].record.op_id.clone();
    assert_ne!(second, OPEN_PUSH_OP_ID);
    let origin = Some("origin".to_string());
    let not_moved = OpResult::NotMoved { refs: vec![("refs/heads/main".to_string(), Some(fx.base.clone()))] };
    assert_eq!(
        push_ops(&fx),
        vec![
            (OPEN_PUSH_OP_ID.to_string(), OpPhase::Intent, None, origin.clone()),
            (OPEN_PUSH_OP_ID.to_string(), OpPhase::Failed, Some(not_moved), origin.clone()),
            (second.clone(), OpPhase::Intent, None, origin.clone()),
            (second, OpPhase::Done, Some(OpResult::Moved), origin),
        ]
    );
    assert_eq!(fx.remote_main(), Some(squash.commit.clone()));
    assert_eq!(fx.branch("main"), squash.commit);
}

#[test]
fn an_open_push_intent_with_the_remote_past_new_is_closed_done() {
    let (fx, squash) = Fixture::landed();
    fx.open_push_intent(&squash.commit);
    fx.push_remote_main(&squash.commit);
    let past = fx.someone_else_pushes_on(&squash.commit, &PAST_SQUASH_FILES, "main");
    assert_open_after_recover(&fx);
    assert_eq!(publish(&fx.git, &refs(), ACTOR).unwrap(), PublishAnswer::Published { origin: squash.commit.clone() });
    let origin = Some("origin".to_string());
    assert_eq!(
        push_ops(&fx),
        vec![
            (OPEN_PUSH_OP_ID.to_string(), OpPhase::Intent, None, origin.clone()),
            (OPEN_PUSH_OP_ID.to_string(), OpPhase::Done, Some(OpResult::Moved), origin),
        ]
    );
    assert_eq!(fx.branch("main"), squash.commit);
    assert_eq!(fx.remote_main(), Some(past));
}

#[test]
fn a_remote_already_past_new_with_no_open_intent_is_published() {
    let (fx, squash) = Fixture::landed();
    fx.push_remote_main(&squash.commit);
    let past = fx.someone_else_pushes_on(&squash.commit, &PAST_SQUASH_FILES, "main");
    assert_eq!(fx.branch("main"), fx.base);
    assert_eq!(publish(&fx.git, &refs(), ACTOR).unwrap(), PublishAnswer::Published { origin: squash.commit.clone() });
    assert_eq!(fx.branch("main"), squash.commit);
    assert_eq!(fx.branch("integration"), squash.commit);
    assert_eq!(fx.landing_json(), None);
    assert_eq!(fx.git.read_ref(SQUASH_REF).unwrap(), None);
    assert_eq!(fx.remote_main(), Some(past));
    assert_eq!(fx.push_records(), vec![]);
    let records = read_op_log(&fx.common).unwrap().records;
    let phases: Vec<(&str, OpPhase)> = records.iter().map(|record| (record.kind.as_str(), record.phase)).collect();
    assert_eq!(
        phases,
        vec![
            ("land", OpPhase::Intent),
            ("land", OpPhase::Done),
            ("publish-origin", OpPhase::Intent),
            ("publish-origin", OpPhase::Done),
            ("publish-squash", OpPhase::Intent),
            ("publish-squash", OpPhase::Done),
        ]
    );
}

/// oplog::recover appends nothing for the fixture's push intent, which stays open.
fn assert_open_after_recover(fx: &Fixture) {
    let recovered = recover(&fx.git, &fx.common).unwrap();
    assert_eq!(recovered.iter().filter(|record| record.kind == "publish-push").count(), 0, "{recovered:?}");
    let records = read_op_log(&fx.common).unwrap().records;
    let open: Vec<&str> = open_intents(&records).iter().map(|record| record.op_id.as_str()).collect();
    assert_eq!(open, vec![OPEN_PUSH_OP_ID]);
}

/// The publish-push records in log order: op id, phase, result and remote mark.
fn push_ops(fx: &Fixture) -> Vec<(String, OpPhase, Option<OpResult>, Option<String>)> {
    fx.push_records()
        .into_iter()
        .map(|marked| (marked.record.op_id, marked.record.phase, marked.record.result, marked.remote))
        .collect()
}
