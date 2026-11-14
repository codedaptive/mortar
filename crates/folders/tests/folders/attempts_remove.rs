//! remove of a request folder against its attempts in a throwaway repository: the AttemptsRemain cases
//! src/remove/attempts_refusal.rs's header names (S-82), an orphan attempt folder left (remove refuses until
//! remake_orphans and keep or drop_attempts clear it; attempts_keep.rs and attempts_drop.rs cover that), and remove of
//! an attempt folder itself.
use super::attempt_support::{try_ref, Attempts, REQ_REF};
use super::maintenance_support::{caller, ACTOR};
use crate::support::run;
use folders::FolderError;

const HAND_MADE: &str = "refs/heads/try/red/one/01";

fn assert_attempts_remain(a: &Attempts, attempts: Vec<String>) {
    let before = a.state();
    match a.remove_request() {
        Err(FolderError::AttemptsRemain { path, attempts: actual }) => {
            assert_eq!(path, a.request);
            assert_eq!(actual, attempts);
        }
        other => panic!("expected AttemptsRemain, got {other:?}"),
    }
    assert_eq!(a.state(), before);
    assert!(a.request.exists());
}

fn assert_removed(a: &Attempts) {
    assert!(!a.request.exists());
    assert_eq!(a.f.git.read_ref(REQ_REF).unwrap(), None);
}

#[test]
fn an_attempt_branch_left_with_no_folder_is_attempts_remain_naming_it_with_nothing_changed() {
    let a = Attempts::with(1);
    run(&a.f.git, &["worktree", "remove", a.folder(1).to_str().unwrap()]);
    assert_attempts_remain(&a, vec![try_ref(1)]);
}

#[test]
fn an_attempt_folder_left_on_its_branch_is_attempts_remain_naming_the_branch_with_nothing_changed() {
    let a = Attempts::with(1);
    assert_attempts_remain(&a, vec![try_ref(1)]);
    assert!(a.folder(1).exists());
}

#[test]
fn only_a_hand_made_try_ref_that_is_not_an_attempt_lets_remove_go_ahead_and_leaves_that_ref() {
    let a = Attempts::new();
    run(&a.f.git, &["update-ref", HAND_MADE, a.f.head.as_str()]);
    a.remove_request().unwrap();
    assert_removed(&a);
    assert_eq!(a.f.git.read_ref(HAND_MADE).unwrap(), Some(a.f.head.clone()));
}

#[test]
fn after_keep_remove_goes_ahead() {
    let a = Attempts::with(3);
    let head = a.f.head.clone();
    assert_eq!(a.keep(&a.f.git, 2, &[(1, true), (3, true)]).unwrap(), (head.clone(), head));
    a.remove_request().unwrap();
    assert_removed(&a);
}

#[test]
fn after_drop_attempts_remove_goes_ahead() {
    let a = Attempts::with(3);
    let saved = [(1, true), (2, true), (3, true)];
    let dropped = caller(&a.f.git, &a.f.common, || folders::drop_attempts(&a.f.git, "red", "one", &saved, ACTOR));
    assert_eq!(dropped.unwrap(), vec![1, 2, 3]);
    a.remove_request().unwrap();
    assert_removed(&a);
}

#[test]
fn an_orphan_attempt_folder_left_is_attempts_remain_naming_its_path() {
    let a = Attempts::with(1);
    a.orphan(1);
    assert_attempts_remain(&a, vec![a.folder(1).display().to_string()]);
    assert!(a.folder(1).exists());
}

#[test]
fn remove_of_an_attempt_folder_is_attempt_folder_with_nothing_changed() {
    let a = Attempts::with(1);
    let before = a.state();
    match caller(&a.f.git, &a.f.common, || folders::remove(&a.f.git, &a.folder(1), true, &[], None, ACTOR)) {
        Err(FolderError::AttemptFolder { path, k }) => assert_eq!((path, k), (a.folder(1), 1)),
        other => panic!("expected AttemptFolder, got {other:?}"),
    }
    assert_eq!(a.state(), before);
    assert!(a.folder(1).exists());
}
