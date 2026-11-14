//! attempt_refs_left_text: the moved ref and where it is, branches kept, the removed folders or none, the next step.

use std::path::PathBuf;

use gitio::Oid;

use super::attempt_refs_left_text::attempt_refs_left_text;

#[test]
fn names_the_ref_where_it_is_and_every_removed_folder() {
    let at = Oid::parse("2222222222222222222222222222222222222222").unwrap();
    let removed = [PathBuf::from("/wt/try-1"), PathBuf::from("/wt/try-2")];
    assert_eq!(
        attempt_refs_left_text("refs/heads/try/folders/WO-1/1", Some(&at), &removed),
        "ref refs/heads/try/folders/WO-1/1 moved (now at 2222222222222222222222222222222222222222) before the attempt \
         branches were deleted; every attempt branch is kept; the attempt folders /wt/try-1, /wt/try-2 were already \
         removed; next step: run the same command again with the same saved attempts"
    );
}

#[test]
fn with_no_folder_removed_says_so() {
    let at = Oid::parse("2222222222222222222222222222222222222222").unwrap();
    assert_eq!(
        attempt_refs_left_text("refs/heads/req/folders/WO-1", Some(&at), &[]),
        "ref refs/heads/req/folders/WO-1 moved (now at 2222222222222222222222222222222222222222) before the attempt \
         branches were deleted; every attempt branch is kept; no attempt folder was removed; next step: run the same \
         command again with the same saved attempts"
    );
}

#[test]
fn an_absent_ref_reads_no_commit() {
    let removed = [PathBuf::from("/wt/try-3")];
    assert_eq!(
        attempt_refs_left_text("refs/heads/try/folders/WO-1/3", None, &removed),
        "ref refs/heads/try/folders/WO-1/3 moved (now at no commit) before the attempt branches were deleted; every \
         attempt branch is kept; the attempt folders /wt/try-3 were already removed; next step: run the same command \
         again with the same saved attempts"
    );
}
