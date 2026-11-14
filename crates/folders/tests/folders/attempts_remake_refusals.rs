//! remake_orphans refusals in a throwaway repository beyond attempts_remake.rs's (S-96 (2), S-98): an orphan whose
//! folder has lost its Mortar folder record is NotAFolder naming that folder, with nothing changed.
use super::attempt_support::{trysnap_ref, try_ref, Attempts};
use crate::support::run;
use folders::record::record_path;
use folders::FolderError;
use std::fs;

#[test]
fn an_orphan_with_a_trysnap_but_no_folder_record_is_not_a_folder_naming_it_with_nothing_changed() {
    let a = Attempts::with(1);
    let snap = a.trysnap(1, true);
    a.orphan(1);
    fs::remove_file(record_path(&a.f.git.git_dir(&a.folder(1)).unwrap())).unwrap();
    let before = a.state();
    let local = a.local(1);
    let index = run(&local, &["ls-files", "--stage", "-z"]);
    let status = run(&local, &["status", "--porcelain", "-z"]);
    match a.remake() {
        Err(FolderError::NotAFolder { path, .. }) => assert_eq!(path, a.folder(1)),
        other => panic!("expected NotAFolder, got {other:?}"),
    }
    assert_eq!(a.f.git.read_ref(&try_ref(1)).unwrap(), None);
    assert_eq!(a.f.git.read_ref(&trysnap_ref(1)).unwrap(), Some(snap));
    assert_eq!(a.state(), before);
    assert_eq!(run(&local, &["ls-files", "--stage", "-z"]), index);
    assert_eq!(run(&local, &["status", "--porcelain", "-z"]), status);
}
