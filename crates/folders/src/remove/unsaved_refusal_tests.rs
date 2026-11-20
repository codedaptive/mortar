//! unsaved_refusal: only saved with no refused file goes ahead; unsaved, refused files, or both, refuse.

use std::path::{Path, PathBuf};

use super::unsaved_refusal::unsaved_refusal;
use crate::make::FolderError;

const FOLDER: &str = "/wt/req-a";

fn refusal(saved: bool, refused: &[PathBuf]) -> Option<(PathBuf, Vec<PathBuf>, Option<u64>)> {
    match unsaved_refusal(Path::new(FOLDER), saved, refused, Some(4096)) {
        None => None,
        Some(FolderError::NotRemovedUnsaved { path, refused, cap }) => Some((path, refused, cap)),
        Some(other) => panic!("expected NotRemovedUnsaved, got {other:?}"),
    }
}

#[test]
fn saved_with_no_refused_file_goes_ahead() {
    assert!(refusal(true, &[]).is_none());
}

#[test]
fn unsaved_with_no_refused_file_refuses() {
    assert_eq!(refusal(false, &[]), Some((PathBuf::from("/wt/req-a"), vec![], Some(4096))));
}

#[test]
fn saved_with_a_refused_file_refuses_naming_it() {
    let refused = [PathBuf::from("big.bin")];
    assert_eq!(
        refusal(true, &refused),
        Some((PathBuf::from("/wt/req-a"), vec![PathBuf::from("big.bin")], Some(4096)))
    );
}

#[test]
fn unsaved_with_refused_files_refuses_naming_them() {
    let refused = [PathBuf::from("big.bin"), PathBuf::from("data/huge.tar")];
    assert_eq!(
        refusal(false, &refused),
        Some((
            PathBuf::from("/wt/req-a"),
            vec![PathBuf::from("big.bin"), PathBuf::from("data/huge.tar")],
            Some(4096)
        ))
    );
}
