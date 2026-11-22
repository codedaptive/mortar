//! create_with_redraw: a taken name is drawn again up to eight draws, a free one is returned with its value, and any
//! other failure stops at once. Closures only: the paths are plain names from a counter and no file is opened.

use std::cell::Cell;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use super::create_with_redraw::create_with_redraw;
use super::GuardError;

/// A draw closure that numbers its paths from 1 and counts its calls in `draws`.
fn counting_draw(draws: &Cell<usize>) -> impl FnMut() -> PathBuf + '_ {
    move || {
        draws.set(draws.get() + 1);
        PathBuf::from(format!("/nowhere/f.mortar-tmp-{}", draws.get()))
    }
}

/// The path and error kind of an Io error; fails the test on anything else.
fn io_parts<T: std::fmt::Debug>(result: Result<(PathBuf, T), GuardError>) -> (PathBuf, ErrorKind) {
    match result {
        Err(GuardError::Io { path, error }) => (path, error.kind()),
        other => panic!("expected GuardError::Io, got {other:?}"),
    }
}

#[test]
fn every_name_taken_gives_up_after_eight_draws_naming_the_eighth() {
    let draws = Cell::new(0);
    let opens = Cell::new(0);
    let result = create_with_redraw(counting_draw(&draws), |_: &Path| -> io::Result<()> {
        opens.set(opens.get() + 1);
        Err(io::Error::from(ErrorKind::AlreadyExists))
    });
    let (path, kind) = io_parts(result);
    assert_eq!(draws.get(), 8);
    assert_eq!(opens.get(), 8);
    assert_eq!(path, PathBuf::from("/nowhere/f.mortar-tmp-8"));
    assert_eq!(kind, ErrorKind::AlreadyExists);
}

#[test]
fn first_name_taken_then_free_returns_the_second_path_and_its_value() {
    let draws = Cell::new(0);
    let result = create_with_redraw(counting_draw(&draws), |path: &Path| {
        if path == Path::new("/nowhere/f.mortar-tmp-1") {
            Err(io::Error::from(ErrorKind::AlreadyExists))
        } else {
            Ok(42)
        }
    });
    assert_eq!(draws.get(), 2);
    assert_eq!(result.ok(), Some((PathBuf::from("/nowhere/f.mortar-tmp-2"), 42)));
}

#[test]
fn any_other_failure_stops_at_the_first_draw() {
    let draws = Cell::new(0);
    let result = create_with_redraw(counting_draw(&draws), |_: &Path| -> io::Result<()> {
        Err(io::Error::from(ErrorKind::PermissionDenied))
    });
    let (path, kind) = io_parts(result);
    assert_eq!(draws.get(), 1);
    assert_eq!(path, PathBuf::from("/nowhere/f.mortar-tmp-1"));
    assert_eq!(kind, ErrorKind::PermissionDenied);
}
