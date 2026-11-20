//! fresh_temp: up to 8 names, a taken one moves on, any other error stops at once, 8 taken is AlreadyExists.

use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use super::fresh_temp;

const TARGET: &str = "/h/.local/bin/mortar";

fn name(hex: &str) -> PathBuf {
    PathBuf::from(format!("/h/.local/bin/.mortar.{hex}.mortar-install"))
}

/// Runs fresh_temp with tokens 1, 2, ... (a 9th token panics) and a `make` that answers `answer` for each path it
/// is given. Returns the result, the paths `make` was asked for and how many tokens were drawn.
fn run(answer: impl Fn(&Path) -> io::Result<()>) -> (io::Result<(PathBuf, ())>, Vec<PathBuf>, u64) {
    let mut drawn = 0;
    let mut asked = Vec::new();
    let result = fresh_temp(
        Path::new(TARGET),
        &mut || {
            assert!(drawn < 8, "a ninth token was drawn");
            drawn += 1;
            drawn
        },
        &mut |path| {
            asked.push(path.to_path_buf());
            answer(path)
        },
    );
    (result, asked, drawn)
}

/// The names for tokens 1 to `count`, written out.
fn taken(count: usize) -> Vec<PathBuf> {
    let all = [
        "0000000000000001",
        "0000000000000002",
        "0000000000000003",
        "0000000000000004",
        "0000000000000005",
        "0000000000000006",
        "0000000000000007",
        "0000000000000008",
    ];
    all[..count].iter().map(|hex| name(hex)).collect()
}

/// A `make` that answers AlreadyExists for a name in `taken` and creates (Ok) any other.
fn taken_answer(taken: Vec<PathBuf>) -> impl Fn(&Path) -> io::Result<()> {
    move |path| match taken.iter().any(|name| name == path) {
        true => Err(io::Error::from(ErrorKind::AlreadyExists)),
        false => Ok(()),
    }
}

#[test]
fn seven_taken_names_then_a_free_one_gives_the_eighth() {
    let (result, asked, drawn) = run(taken_answer(taken(7)));

    assert_eq!(result.unwrap().0, name("0000000000000008"));
    assert_eq!(asked.len(), 8);
    assert_eq!(asked[0], name("0000000000000001"));
    assert_eq!(asked[7], name("0000000000000008"));
    assert_eq!(drawn, 8);
}

#[test]
fn eight_taken_names_give_already_exists_naming_the_path() {
    let (result, asked, drawn) = run(taken_answer(taken(8)));
    let error = result.unwrap_err();

    assert_eq!(error.kind(), ErrorKind::AlreadyExists);
    assert!(error.to_string().contains("/h/.local/bin/mortar"), "{error}");
    assert_eq!(asked.len(), 8);
    assert_eq!(drawn, 8);
}

#[test]
fn another_error_is_returned_at_once() {
    let (result, asked, drawn) = run(|_| Err(io::Error::from(ErrorKind::PermissionDenied)));

    assert_eq!(result.unwrap_err().kind(), ErrorKind::PermissionDenied);
    assert_eq!(asked, vec![name("0000000000000001")]);
    assert_eq!(drawn, 1);
}

#[test]
fn a_free_first_name_is_taken_after_one_try() {
    let (result, asked, drawn) = run(|_| Ok(()));

    assert_eq!(result.unwrap().0, name("0000000000000001"));
    assert_eq!(asked, vec![name("0000000000000001")]);
    assert_eq!(drawn, 1);
}
