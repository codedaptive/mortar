//! with_object_quarantine: the closure's git adds the two object env pairs to the caller's, with a fresh directory
//! under the OS temp dir that exists while the closure runs and is gone after, also after a panic. No git runs.

use std::ffi::OsString;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use gitio::Git;

use super::with_object_quarantine;

const OBJECTS: &str = "/r/.git/objects";

fn git() -> Git {
    Git {
        program: PathBuf::from("git"),
        repo: PathBuf::from("/r/folder"),
        env: vec![(OsString::from("GIT_AUTHOR_NAME"), OsString::from("someone"))],
    }
}

fn value<'a>(git: &'a Git, name: &str) -> Vec<&'a OsString> {
    git.env.iter().filter(|(key, _)| key == name).map(|(_, value)| value).collect()
}

#[test]
fn the_closure_git_keeps_the_callers_and_adds_the_object_dir_and_the_alternate() {
    let (seen, dir) = with_object_quarantine(&git(), Path::new(OBJECTS), |quarantined| {
        let dir = PathBuf::from(value(quarantined, "GIT_OBJECT_DIRECTORY")[0]);
        (quarantined.clone(), dir)
    })
    .unwrap();
    assert_eq!(seen.program, PathBuf::from("git"));
    assert_eq!(seen.repo, PathBuf::from("/r/folder"));
    assert_eq!(seen.env.len(), 3);
    assert_eq!(seen.env[0], (OsString::from("GIT_AUTHOR_NAME"), OsString::from("someone")));
    assert_eq!(value(&seen, "GIT_ALTERNATE_OBJECT_DIRECTORIES"), vec![&OsString::from(OBJECTS)]);
    assert_eq!(dir.parent(), Some(std::env::temp_dir().as_path()));
}

#[test]
fn the_directory_exists_inside_the_closure_and_is_gone_after() {
    let dir = with_object_quarantine(&git(), Path::new(OBJECTS), |quarantined| {
        let dir = PathBuf::from(value(quarantined, "GIT_OBJECT_DIRECTORY")[0]);
        assert!(dir.is_dir(), "{dir:?}");
        std::fs::write(dir.join("written"), b"an object").unwrap();
        dir
    })
    .unwrap();
    assert!(!dir.exists(), "{dir:?}");
}

#[test]
fn the_directory_is_gone_after_a_panic_in_the_closure() {
    let seen: Mutex<Option<PathBuf>> = Mutex::new(None);
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        with_object_quarantine(&git(), Path::new(OBJECTS), |quarantined| {
            let dir = PathBuf::from(value(quarantined, "GIT_OBJECT_DIRECTORY")[0]);
            assert!(dir.is_dir());
            *seen.lock().unwrap() = Some(dir);
            panic!("the closure fails");
        })
    }));
    assert!(outcome.is_err());
    let dir = seen.lock().unwrap().clone().unwrap();
    assert!(!dir.exists(), "{dir:?}");
}

#[test]
fn two_quarantines_get_two_directories() {
    let dir = || {
        with_object_quarantine(&git(), Path::new(OBJECTS), |quarantined| {
            PathBuf::from(value(quarantined, "GIT_OBJECT_DIRECTORY")[0])
        })
        .unwrap()
    };
    assert_ne!(dir(), dir());
}
