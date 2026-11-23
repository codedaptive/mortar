use crate::fixture::{refs, Fixture, ACTOR, ID, RECEIPT};
use land::{back_out, commit_files, publish, BackOutAnswer, CommitFilesAnswer, PublishAnswer};
use oplog::{ChangeId, OpPhase};
use std::path::PathBuf;

#[test]
fn named_commit_is_published_without_the_unnamed_edit() {
    let (fx, squash, git) = Fixture::landed();
    fx.receipt();
    std::fs::write(fx.folder.join("a.txt"), "uncommitted\n").unwrap();
    let before = oplog::read_op_log(&fx.common).unwrap().records;
    let CommitFilesAnswer::Committed { commit } = commit_files(&git, &refs(), &[PathBuf::from(RECEIPT)], "receipt\n", ACTOR).unwrap() else {
        panic!("named receipt must be committed");
    };
    assert_eq!(fx.branch("integration"), commit);
    fx.assert_commit(&commit, &squash.commit);
    assert_eq!(std::fs::read_to_string(fx.folder.join("a.txt")).unwrap(), "uncommitted\n");
    let after = oplog::read_op_log(&fx.common).unwrap().records;
    assert_eq!(after.len(), before.len() + 2);
    assert_eq!(fx.phases(), vec![OpPhase::Intent, OpPhase::Done]);
    assert_eq!(fx.state(), Some(fx.state_text(&squash.commit, &commit, None)));
    assert_eq!(publish(&git, &refs(), ACTOR).unwrap(), PublishAnswer::Published { origin: commit.clone() });
    assert_eq!(fx.branch("main"), commit);
    assert_eq!(fx.remote.read_ref("refs/heads/main").unwrap(), Some(commit));
}

#[test]
fn back_out_drops_the_landing_and_named_commit() {
    let (fx, squash, git) = Fixture::landed();
    fx.receipt();
    let CommitFilesAnswer::Committed { commit } = commit_files(&git, &refs(), &[PathBuf::from(RECEIPT)], "receipt\n", ACTOR).unwrap() else {
        panic!("named receipt must be committed");
    };
    fx.assert_commit(&commit, &squash.commit);
    assert_eq!(back_out(&git, &refs(), ACTOR).unwrap(), BackOutAnswer::BackedOut { change_id: ChangeId::parse(ID).unwrap() });
    assert_eq!(fx.branch("integration"), fx.base);
    assert_eq!(fx.state(), None);
}

#[test]
fn same_content_writes_no_op_or_state() {
    let (fx, squash, git) = Fixture::landed();
    let log = fx.log();
    let state = fx.state();
    assert_eq!(commit_files(&git, &refs(), &[PathBuf::from("a.txt")], "same\n", ACTOR).unwrap(), CommitFilesAnswer::Same);
    assert_eq!(fx.branch("integration"), squash.commit);
    assert_eq!(fx.log(), log);
    assert_eq!(fx.state(), state);
}
