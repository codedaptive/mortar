use crate::fake_call::FakeCall;
use crate::replay_git_fake::ReplayGitFake;
use crate::replay_in::replay_in;
use crate::{JoinError, ReplayResult};
use gitio::{CommitInfo, MergeTreeOutcome, Oid, Person, RefUpdate, ReplayOutcome};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::PathBuf;

const BASE: &str = "1111111111111111111111111111111111111111";
const ONTO: &str = "3333333333333333333333333333333333333333";
const TIP_OID: &str = "2222222222222222222222222222222222222222";
const TIP: &str = "refs/heads/topic";
const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TREE_A: &str = "a111111111111111111111111111111111111111";
const NEW: &str = "9999999999999999999999999999999999999999";
const ONTO_TREE: &str = "0000000000000000000000000000000000000001";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

/// onto present (tree ONTO_TREE, which no merge gives, so A never drops), the tip ref at TIP_OID, and a range of
/// one commit A (parent BASE) whose merge is `merge`.
fn fake(available: bool, merge: MergeTreeOutcome) -> ReplayGitFake {
    let mut git = ReplayGitFake {
        available: Some(available),
        rev_parse: Some(Ok(Some(oid(ONTO)))),
        rev_list: Some(vec![oid(A)]),
        new_commits: RefCell::new(VecDeque::from([oid(NEW)])),
        ..ReplayGitFake::default()
    };
    git.refs.insert(TIP.to_string(), oid(TIP_OID));
    let author = Person { name: "Bo".to_string(), email: "bo@example.com".to_string(), date: None };
    let info = |tree: &str, message: &str| CommitInfo {
        tree: oid(tree),
        parents: vec![oid(BASE)],
        author: author.clone(),
        message: message.to_string(),
    };
    git.commits.insert(oid(A), info(TIP_OID, "a\n"));
    git.commits.insert(oid(ONTO), info(ONTO_TREE, "onto\n"));
    git.merges.insert(oid(A), merge);
    git
}

fn calls(git: &ReplayGitFake) -> Vec<FakeCall> {
    git.calls.borrow().clone()
}

#[test]
fn missing_onto_is_refused_after_only_the_rev_parse() {
    let git = ReplayGitFake { rev_parse: Some(Ok(None)), ..fake(true, MergeTreeOutcome::Clean(oid(TREE_A))) };
    assert_eq!(replay_in(&git, &oid(BASE), TIP, &oid(ONTO)), Err(JoinError::OntoMissing(oid(ONTO))));
    assert_eq!(calls(&git), vec![FakeCall::RevParse(ONTO.to_string())]);
}

#[test]
fn missing_tip_is_refused_before_any_replay_or_merge() {
    let mut git = fake(true, MergeTreeOutcome::Clean(oid(TREE_A)));
    git.refs.clear();
    assert_eq!(
        replay_in(&git, &oid(BASE), TIP, &oid(ONTO)),
        Err(JoinError::TipMissing("refs/heads/topic".to_string()))
    );
    assert_eq!(
        calls(&git),
        vec![FakeCall::RevParse(ONTO.to_string()), FakeCall::ReadRef("refs/heads/topic".to_string())]
    );
}

#[test]
fn available_and_clean_gives_git_replays_tip_without_merging() {
    let update = RefUpdate { refname: TIP.to_string(), new: oid(NEW), old: Some(oid(TIP_OID)) };
    let git = ReplayGitFake {
        replay: Some(ReplayOutcome::Replayed(vec![update])),
        ..fake(true, MergeTreeOutcome::Clean(oid(TREE_A)))
    };
    assert_eq!(replay_in(&git, &oid(BASE), TIP, &oid(ONTO)), Ok(ReplayResult::Clean(oid(NEW))));
    assert_eq!(
        calls(&git),
        vec![
            FakeCall::RevParse(ONTO.to_string()),
            FakeCall::ReadRef("refs/heads/topic".to_string()),
            FakeCall::ReplayAvailable,
            FakeCall::Replay { onto: oid(ONTO), base: oid(BASE), tip_ref: "refs/heads/topic".to_string() },
        ]
    );
}

#[test]
fn available_and_conflicted_returns_the_fallbacks_conflict() {
    let conflict = MergeTreeOutcome::Conflicted { tree: oid(TREE_A), files: vec![PathBuf::from("x.txt")] };
    let git = ReplayGitFake { replay: Some(ReplayOutcome::Conflicted), ..fake(true, conflict) };
    assert_eq!(
        replay_in(&git, &oid(BASE), TIP, &oid(ONTO)),
        Ok(ReplayResult::Conflicted {
            commit: oid(A),
            replayed_tip: oid(ONTO),
            remaining: Vec::new(),
            tree: oid(TREE_A),
            files: vec![PathBuf::from("x.txt")],
        })
    );
    assert_eq!(calls(&git)[4], FakeCall::RevList { base: oid(BASE), tip: oid(TIP_OID) });
}

#[test]
fn unavailable_skips_git_replay_and_returns_the_fallbacks_tip() {
    let git = fake(false, MergeTreeOutcome::Clean(oid(TREE_A)));
    assert_eq!(replay_in(&git, &oid(BASE), TIP, &oid(ONTO)), Ok(ReplayResult::Clean(oid(NEW))));
    let calls = calls(&git);
    assert_eq!(calls[2], FakeCall::ReplayAvailable);
    assert_eq!(calls[3], FakeCall::RevList { base: oid(BASE), tip: oid(TIP_OID) });
    assert!(!calls.iter().any(|call| matches!(call, FakeCall::Replay { .. })));
}

#[test]
fn available_with_no_updates_runs_the_fallback_and_returns_its_tip() {
    let git = ReplayGitFake {
        replay: Some(ReplayOutcome::Replayed(Vec::new())),
        ..fake(true, MergeTreeOutcome::Clean(oid(TREE_A)))
    };
    assert_eq!(replay_in(&git, &oid(BASE), TIP, &oid(ONTO)), Ok(ReplayResult::Clean(oid(NEW))));
    assert_eq!(
        calls(&git),
        vec![
            FakeCall::RevParse(ONTO.to_string()),
            FakeCall::ReadRef("refs/heads/topic".to_string()),
            FakeCall::ReplayAvailable,
            FakeCall::Replay { onto: oid(ONTO), base: oid(BASE), tip_ref: "refs/heads/topic".to_string() },
            FakeCall::RevList { base: oid(BASE), tip: oid(TIP_OID) },
            FakeCall::CommitInfo(oid(A)),
            FakeCall::MergeTree { attr_source: oid(ONTO), merge_base: Some(oid(BASE)), ours: oid(ONTO), theirs: oid(A) },
            FakeCall::CommitInfo(oid(ONTO)),
            FakeCall::CommitTree {
                tree: oid(TREE_A),
                parents: vec![oid(ONTO)],
                message: "a\n".to_string(),
                author: Some(Person { name: "Bo".to_string(), email: "bo@example.com".to_string(), date: None }),
                committer: None,
            },
        ]
    );
}
