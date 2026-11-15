//! map_steps: the replay loop's decisions over scripted steps; no git.

use crate::map_steps::map_steps;
use crate::step::Step;
use crate::{JoinError, MappedCommit, MappedReplay, ReplayResult};
use gitio::Oid;
use std::collections::VecDeque;
use std::path::PathBuf;

const START: &str = "1111111111111111111111111111111111111111";
const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const C: &str = "cccccccccccccccccccccccccccccccccccccccc";
const NEW_A: &str = "a000000000000000000000000000000000000001";
const NEW_B: &str = "b000000000000000000000000000000000000002";
const NEW_C: &str = "c000000000000000000000000000000000000003";
const TREE: &str = "7777777777777777777777777777777777777777";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn entry(source: &str, replayed: &str) -> MappedCommit {
    MappedCommit { source: oid(source), replayed: oid(replayed) }
}

/// Runs map_steps over A, B and C from START, answering each call with the next of `steps`; gives the answer and
/// each call's (running tip, commit).
fn run(steps: Vec<Result<Step, JoinError>>) -> (Result<MappedReplay, JoinError>, Vec<(Oid, Oid)>) {
    let mut steps = VecDeque::from(steps);
    let mut seen = Vec::new();
    let answer = map_steps(&oid(START), &[oid(A), oid(B), oid(C)], |running, commit| {
        seen.push((running.clone(), commit.clone()));
        steps.pop_front().expect("a step past the script")
    });
    (answer, seen)
}

#[test]
fn an_empty_list_is_clean_at_start_with_no_entries_and_no_step() {
    let mut called = false;
    let answer = map_steps(&oid(START), &[], |_, _| {
        called = true;
        Ok(Step::Dropped)
    });
    assert_eq!(answer, Ok(MappedReplay { result: ReplayResult::Clean(oid(START)), mapped: Vec::new() }));
    assert!(!called);
}

#[test]
fn clean_steps_map_each_commit_to_its_new_commit_in_order_and_chain_the_running_tip() {
    let steps = vec![Ok(Step::Clean(oid(NEW_A))), Ok(Step::Clean(oid(NEW_B))), Ok(Step::Clean(oid(NEW_C)))];
    let (answer, seen) = run(steps);
    assert_eq!(
        answer,
        Ok(MappedReplay {
            result: ReplayResult::Clean(oid(NEW_C)),
            mapped: vec![entry(A, NEW_A), entry(B, NEW_B), entry(C, NEW_C)],
        })
    );
    assert_eq!(seen, vec![(oid(START), oid(A)), (oid(NEW_A), oid(B)), (oid(NEW_B), oid(C))]);
}

#[test]
fn a_dropped_middle_commit_maps_to_the_previous_new_commit() {
    let (answer, seen) = run(vec![Ok(Step::Clean(oid(NEW_A))), Ok(Step::Dropped), Ok(Step::Clean(oid(NEW_C)))]);
    assert_eq!(
        answer,
        Ok(MappedReplay {
            result: ReplayResult::Clean(oid(NEW_C)),
            mapped: vec![entry(A, NEW_A), entry(B, NEW_A), entry(C, NEW_C)],
        })
    );
    assert_eq!(seen, vec![(oid(START), oid(A)), (oid(NEW_A), oid(B)), (oid(NEW_A), oid(C))]);
}

#[test]
fn a_dropped_first_commit_maps_to_start() {
    let (answer, seen) = run(vec![Ok(Step::Dropped), Ok(Step::Clean(oid(NEW_B))), Ok(Step::Clean(oid(NEW_C)))]);
    assert_eq!(
        answer,
        Ok(MappedReplay {
            result: ReplayResult::Clean(oid(NEW_C)),
            mapped: vec![entry(A, START), entry(B, NEW_B), entry(C, NEW_C)],
        })
    );
    assert_eq!(seen, vec![(oid(START), oid(A)), (oid(START), oid(B)), (oid(NEW_B), oid(C))]);
}

#[test]
fn every_commit_dropped_is_clean_at_start_with_every_entry_start() {
    let (answer, seen) = run(vec![Ok(Step::Dropped), Ok(Step::Dropped), Ok(Step::Dropped)]);
    assert_eq!(
        answer,
        Ok(MappedReplay {
            result: ReplayResult::Clean(oid(START)),
            mapped: vec![entry(A, START), entry(B, START), entry(C, START)],
        })
    );
    assert_eq!(seen, vec![(oid(START), oid(A)), (oid(START), oid(B)), (oid(START), oid(C))]);
}

#[test]
fn a_dropped_last_commit_leaves_the_previous_new_commit_as_the_tip() {
    let (answer, _) = run(vec![Ok(Step::Clean(oid(NEW_A))), Ok(Step::Clean(oid(NEW_B))), Ok(Step::Dropped)]);
    assert_eq!(
        answer,
        Ok(MappedReplay {
            result: ReplayResult::Clean(oid(NEW_B)),
            mapped: vec![entry(A, NEW_A), entry(B, NEW_B), entry(C, NEW_B)],
        })
    );
}

#[test]
fn a_conflict_at_the_second_maps_only_the_first_and_stops() {
    let conflicted = Step::Conflicted { tree: oid(TREE), files: vec![PathBuf::from("x.txt")] };
    let (answer, seen) = run(vec![Ok(Step::Clean(oid(NEW_A))), Ok(conflicted)]);
    assert_eq!(
        answer,
        Ok(MappedReplay {
            result: ReplayResult::Conflicted {
                commit: oid(B),
                replayed_tip: oid(NEW_A),
                remaining: vec![oid(C)],
                tree: oid(TREE),
                files: vec![PathBuf::from("x.txt")],
            },
            mapped: vec![entry(A, NEW_A)],
        })
    );
    assert_eq!(seen, vec![(oid(START), oid(A)), (oid(NEW_A), oid(B))]);
}

#[test]
fn an_error_from_a_step_returns_at_once() {
    let (answer, seen) = run(vec![Ok(Step::Clean(oid(NEW_A))), Err(JoinError::RootCommit(oid(B)))]);
    assert_eq!(answer, Err(JoinError::RootCommit(oid(B))));
    assert_eq!(seen, vec![(oid(START), oid(A)), (oid(NEW_A), oid(B))]);
}
