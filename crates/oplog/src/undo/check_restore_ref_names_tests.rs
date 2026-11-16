//! check_restore_ref_names: names refused before git runs. The Git points at no binary and no repository, so a
//! git call would return OplogError::Git and fail the test.

use std::path::PathBuf;

use gitio::Git;

use super::check_restore_ref_names;
use crate::store::OplogError;

fn no_git() -> Git {
    Git {
        program: PathBuf::from("/nonexistent/mortar-no-git"),
        ..Git::new("/nonexistent/mortar-no-repo")
    }
}

fn refused_name(refs: &[&str]) -> String {
    let refs: Vec<String> = refs.iter().map(|r| r.to_string()).collect();
    match check_restore_ref_names(&no_git(), &refs) {
        Err(OplogError::BadMove { refname, .. }) => refname,
        other => panic!("expected BadMove before git runs: {other:?}"),
    }
}

#[test]
fn an_empty_name_is_bad_move() {
    assert_eq!(refused_name(&[""]), "");
}

#[test]
fn a_nul_holding_name_is_bad_move() {
    assert_eq!(refused_name(&["refs/heads/a\0b"]), "refs/heads/a\0b");
}

#[test]
fn a_name_outside_refs_is_bad_move() {
    assert_eq!(refused_name(&["heads/main"]), "heads/main");
}

#[test]
fn every_name_is_checked_before_git_runs() {
    assert_eq!(refused_name(&["refs/heads/x", ""]), "");
}
