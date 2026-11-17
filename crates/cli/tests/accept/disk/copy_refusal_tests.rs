use super::{copy_refusal, Registered};
use std::path::{Path, PathBuf};

#[test]
fn an_empty_registration_list_fails_closed() {
    assert_eq!(copy_refusal(Path::new("/copy"), Path::new("/copy/.git"), &[]), Some("no registered team repository was read, so the drive cannot tell /copy is not one".to_string()));
}

#[test]
fn an_exact_registered_path_is_refused_before_any_common_dir_match() {
    let registered = vec![
        Registered { path: PathBuf::from("/other"), common: Some(PathBuf::from("/shared")) },
        Registered { path: PathBuf::from("/copy"), common: None },
    ];
    assert_eq!(copy_refusal(Path::new("/copy"), Path::new("/shared"), &registered), Some("/copy is the registered team repository /copy".to_string()));
}

#[test]
fn a_common_dir_equal_to_the_registered_path_is_refused() {
    let registered = vec![Registered { path: PathBuf::from("/team"), common: None }];
    assert_eq!(copy_refusal(Path::new("/copy"), Path::new("/team"), &registered), Some("/copy is a worktree of the registered team repository /team (git common dir /team)".to_string()));
}

#[test]
fn a_common_dir_equal_to_the_registered_dot_git_is_refused() {
    let registered = vec![Registered { path: PathBuf::from("/team"), common: None }];
    assert_eq!(copy_refusal(Path::new("/copy"), Path::new("/team/.git"), &registered), Some("/copy is a worktree of the registered team repository /team (git common dir /team/.git)".to_string()));
}

#[test]
fn a_matching_recorded_common_dir_is_refused_using_the_first_matching_registration() {
    let registered = vec![
        Registered { path: PathBuf::from("/first"), common: Some(PathBuf::from("/shared")) },
        Registered { path: PathBuf::from("/second"), common: Some(PathBuf::from("/shared")) },
    ];
    assert_eq!(copy_refusal(Path::new("/copy"), Path::new("/shared"), &registered), Some("/copy is a worktree of the registered team repository /first (git common dir /shared)".to_string()));
}

#[test]
fn unrelated_copy_and_common_dir_are_allowed() {
    let registered = vec![Registered { path: PathBuf::from("/team"), common: Some(PathBuf::from("/team/.git")) }];
    assert_eq!(copy_refusal(Path::new("/copy"), Path::new("/copy/.git"), &registered), None);
}
