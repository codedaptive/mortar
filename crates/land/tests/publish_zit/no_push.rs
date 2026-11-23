//! Publish without a remote: fresh landing, cleanup after interruption, and a moved local origin; push-on outcomes.

use crate::fixture::{refs, Fixture, ACTOR, OTHER_FILES, SQUASH_REF};
use gitio::RunOpts;
use land::{publish_with_push, PublishAnswer, PublishOutcome};
use oplog::{read_op_log, OpPhase, RefMove};

#[test]
fn push_off_publishes_locally_without_reading_the_remote() {
    let (fx, squash) = Fixture::landed();
    let missing = fx.elsewhere.path.join("missing-remote.git");
    fx.git.run(&["remote", "set-url", "origin", missing.to_str().unwrap()], &RunOpts::default()).unwrap();

    assert_eq!(
        publish_with_push(&fx.git, &refs(), ACTOR, false).unwrap(),
        PublishOutcome { answer: PublishAnswer::Published { origin: squash.commit.clone() }, pushed: false }
    );
    assert_eq!(fx.branch("main"), squash.commit);
    assert_eq!(fx.remote_main(), Some(fx.base.clone()));
    assert_eq!(fx.landing_json(), None);
    assert_eq!(fx.git.read_ref(SQUASH_REF).unwrap(), None);
    assert!(fx.push_log().is_empty());
    let records = read_op_log(&fx.common).unwrap().records;
    let phases: Vec<_> = records.iter().map(|record| (record.kind.as_str(), record.phase)).collect();
    assert_eq!(phases, vec![
        ("land", OpPhase::Intent), ("land", OpPhase::Done),
        ("publish-origin", OpPhase::Intent), ("publish-origin", OpPhase::Done),
        ("publish-squash", OpPhase::Intent), ("publish-squash", OpPhase::Done),
    ]);
    let origin_move = vec![RefMove {
        refname: "refs/heads/main".to_string(), old: Some(fx.base.clone()), new: squash.commit.clone(),
    }];
    assert_eq!(records[2].moves, origin_move);
    assert_eq!(records[3].moves, origin_move);
}

#[test]
fn push_off_repeat_after_origin_advanced_only_cleans_up() {
    let (fx, squash) = Fixture::landed();
    fx.set_ref("refs/heads/main", &squash.commit);

    assert_eq!(
        publish_with_push(&fx.git, &refs(), ACTOR, false).unwrap(),
        PublishOutcome { answer: PublishAnswer::Published { origin: squash.commit.clone() }, pushed: false }
    );
    assert_eq!(fx.branch("main"), squash.commit);
    assert_eq!(fx.landing_json(), None);
    assert_eq!(fx.git.read_ref(SQUASH_REF).unwrap(), None);
    assert_eq!(fx.remote_main(), Some(fx.base.clone()));
    assert!(fx.push_log().is_empty());
    let records = read_op_log(&fx.common).unwrap().records;
    let phases: Vec<_> = records.iter().map(|record| (record.kind.as_str(), record.phase)).collect();
    assert_eq!(phases, vec![
        ("land", OpPhase::Intent), ("land", OpPhase::Done),
        ("publish-squash", OpPhase::Intent), ("publish-squash", OpPhase::Done),
    ]);
    assert_eq!(
        publish_with_push(&fx.git, &refs(), ACTOR, false).unwrap(),
        PublishOutcome { answer: PublishAnswer::NothingToPublish, pushed: false }
    );
    assert_eq!(read_op_log(&fx.common).unwrap().records.len(), records.len());
}

#[test]
fn push_off_moved_local_origin_backs_out_without_fetching() {
    let (fx, squash) = Fixture::landed();
    let local = fx.squash(&fx.base, &OTHER_FILES).commit;
    fx.set_ref("refs/heads/main", &local);
    let remote = fx.someone_else_pushes("main");
    assert_ne!(local, remote);

    assert_eq!(
        publish_with_push(&fx.git, &refs(), ACTOR, false).unwrap(),
        PublishOutcome { answer: PublishAnswer::OriginMoved { found: local.clone() }, pushed: false }
    );
    assert_eq!(fx.branch("main"), local);
    assert_eq!(fx.branch("integration"), local);
    assert_eq!(fx.landing_json(), None);
    assert_eq!(fx.git.read_ref(SQUASH_REF).unwrap(), Some(squash.commit));
    assert_eq!(fx.remote_main(), Some(remote));
    assert!(fx.push_log().is_empty());
}

#[test]
fn push_on_published_reports_pushed() {
    let (fx, squash) = Fixture::landed();
    assert_eq!(
        publish_with_push(&fx.git, &refs(), ACTOR, true).unwrap(),
        PublishOutcome { answer: PublishAnswer::Published { origin: squash.commit.clone() }, pushed: true }
    );
    assert_eq!(fx.remote_main(), Some(squash.commit));
}

#[test]
fn push_on_moved_remote_reports_not_pushed() {
    let (fx, _) = Fixture::landed();
    let moved = fx.someone_else_pushes("main");
    assert_eq!(
        publish_with_push(&fx.git, &refs(), ACTOR, true).unwrap(),
        PublishOutcome { answer: PublishAnswer::OriginMoved { found: moved.clone() }, pushed: false }
    );
    assert_eq!(fx.remote_main(), Some(moved));
}
