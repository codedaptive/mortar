//! path_kind's answers for paths not written as git stores them, as git_path_kind.rs's doc names them: ls-tree's own
//! answer, not normalised.
use crate::fixture::holding_each_kind;
use gitio::GitError;
use std::path::Path;

#[test]
fn kind_a_file_with_a_trailing_slash_gives_none() {
    let (r, root, _) = holding_each_kind();
    assert_eq!(r.git.path_kind(&root, Path::new("file/")), Ok(None));
}

#[test]
fn kind_a_leading_dot_slash_is_refused_as_another_path() {
    let (r, root, _) = holding_each_kind();
    let Err(GitError { args, code, stderr }) = r.git.path_kind(&root, Path::new("./file")) else {
        panic!("./file was answered, not refused");
    };
    assert_eq!((args.is_empty(), code), (true, None));
    assert!(stderr.starts_with("not one ls-tree record for \"./file\" (another path)"), "{stderr}");
}

#[test]
fn kind_a_folder_with_a_trailing_slash_is_refused() {
    let (r, root, _) = holding_each_kind();
    let Err(GitError { args, code, stderr }) = r.git.path_kind(&root, Path::new("dir/")) else {
        panic!("dir/ was answered, not refused");
    };
    assert_eq!((args.is_empty(), code), (true, None));
    assert!(stderr.starts_with("not one ls-tree record for \"dir/\" (another path)"), "{stderr}");
}

#[test]
fn kind_a_missing_path_with_a_trailing_slash_gives_none() {
    let (r, root, _) = holding_each_kind();
    assert_eq!(r.git.path_kind(&root, Path::new("missing/")), Ok(None));
}

#[test]
fn kind_a_leading_slash_is_gits_own_error() {
    let (r, root, _) = holding_each_kind();
    let answer = r.git.path_kind(&root, Path::new("/file"));
    assert!(matches!(answer, Err(GitError { code: Some(_), .. })), "{answer:?}");
}

#[test]
fn kind_an_empty_path_is_refused() {
    let (r, root, _) = holding_each_kind();
    let Err(GitError { code, stderr, .. }) = r.git.path_kind(&root, Path::new("")) else {
        panic!("the empty path was answered, not refused");
    };
    assert_eq!((code, stderr.as_str()), (None, "not a usable path (empty): \"\""));
}
