//! run_quarantined and quarantine_dir: the remover runs once, after act, whether act returns or panics; a failed make
//! runs neither act nor the remover; the name is drawn from the pid, the count and the time. The make and remove
//! steps are recorders, so no directory is made and no git runs.

use std::cell::RefCell;
use std::ffi::OsString;
use std::io;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};

use gitio::Git;

use super::{quarantine_dir, run_quarantined};

fn git() -> Git {
    Git { program: PathBuf::from("git"), repo: PathBuf::from("/r/folder"), env: Vec::new() }
}

fn run<T>(
    events: &RefCell<Vec<String>>,
    make: impl FnOnce(&Path) -> io::Result<()>,
    act: impl FnOnce(&Git) -> T,
) -> Result<T, String> {
    run_quarantined(
        &git(),
        Path::new("/r/.git/objects"),
        PathBuf::from("/q/dir"),
        make,
        |dir| events.borrow_mut().push(format!("remove {}", dir.display())),
        act,
    )
}

#[test]
fn the_remover_runs_once_after_act_returns_and_the_answer_comes_back() {
    let events = RefCell::new(Vec::new());
    let answer = run(
        &events,
        |_| Ok(()),
        |git| {
            events.borrow_mut().push("act".to_string());
            assert!(git.env.contains(&(OsString::from("GIT_OBJECT_DIRECTORY"), OsString::from("/q/dir"))));
            "the answer"
        },
    );
    assert_eq!(answer, Ok("the answer"));
    assert_eq!(events.into_inner(), vec!["act".to_string(), "remove /q/dir".to_string()]);
}

#[test]
fn the_remover_runs_once_after_act_panics() {
    let events = RefCell::new(Vec::new());
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        run(&events, |_| Ok(()), |_| -> () {
            events.borrow_mut().push("act".to_string());
            panic!("act failed")
        })
    }));
    assert!(outcome.is_err());
    assert_eq!(events.into_inner(), vec!["act".to_string(), "remove /q/dir".to_string()]);
}

#[test]
fn a_failed_make_is_the_err_and_runs_neither_act_nor_the_remover() {
    let events = RefCell::new(Vec::new());
    let answer = run(&events, |_| Err(io::Error::other("no room")), |_| {
        events.borrow_mut().push("act".to_string());
    });
    assert_eq!(answer, Err("the object quarantine /q/dir could not be made: no room".to_string()));
    assert!(events.into_inner().is_empty());
}

#[test]
fn the_name_is_drawn_from_the_pid_the_count_and_the_time() {
    assert_eq!(quarantine_dir(Path::new("/tmp"), 42, 7, 99), PathBuf::from("/tmp/mortar-disk-objects-42-7-99"));
    assert_ne!(quarantine_dir(Path::new("/tmp"), 42, 0, 99), quarantine_dir(Path::new("/tmp"), 42, 1, 99));
}
