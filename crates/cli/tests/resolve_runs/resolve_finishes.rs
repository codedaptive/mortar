use crate::{objects::*, paused::Paused, support::*};
use std::fs;

#[test]
fn clean_resolve_consumes_folder_tree_moves_real_index_files_and_head_preserves_local_work_and_repeats_once() {
    let r = Paused::new(false); r.first(); let f = &r.f;
    let old_info = f.git.commit_info(&r.old).unwrap();
    f.write("src/file", b"chosen resolution\n"); f.write("sub/keep", b"unrelated dirty tracked work\n");
    f.write("src/local-only", b"untracked work must survive\n");
    let expected_tree = tree(f, &r.zone, &[("src/file", b"chosen resolution\n")], &[], &[]);
    let report = f.report(&["--json", "resolve"], "resolve", 0, "rebased"); let new = f.head();
    assert_ne!(new, r.old); assert_eq!(report.text("request_ref"), REQUEST);
    assert_eq!(report.text("from"), r.old.as_str()); assert_eq!(report.text("to"), new.as_str());
    assert_eq!(report.text("next"), "mr join"); assert_eq!(report.text("snapshot_ref"), SNAP);
    assert!(report.is_null("finished_move_from")); assert!(report.is_null("finished_move_to"));
    let snap = f.git.read_ref(SNAP).unwrap().unwrap(); assert_eq!(report.text("snapshot_commit"), snap.as_str());
    assert_eq!(f.blob(&snap, "src/file"), b"chosen resolution\n");
    assert_eq!(f.blob(&snap, "sub/keep"), b"unrelated dirty tracked work\n");
    assert_eq!(f.blob(&snap, "src/local-only"), b"untracked work must survive\n");
    let info = f.git.commit_info(&new).unwrap();
    assert_eq!(info.tree, expected_tree); assert_eq!(info.parents, vec![r.zone.clone()]);
    assert_eq!(info.author, old_info.author); assert_eq!(info.message, old_info.message);
    assert_eq!(f.blob(&new, "sub/keep"), b"keep\n");
    assert!(f.git.tree_entry(&info.tree, "src/local-only".as_ref()).unwrap().is_none());
    assert_eq!(fs::read(f.request.join("src/file")).unwrap(), b"chosen resolution\n");
    assert_eq!(fs::read(f.request.join("src/upstream")).unwrap(), b"advanced zone work\n");
    assert_eq!(fs::read(f.request.join("sub/keep")).unwrap(), b"unrelated dirty tracked work\n");
    assert_eq!(fs::read(f.request.join("src/local-only")).unwrap(), b"untracked work must survive\n");
    assert_eq!(raw(&f.local(), &["write-tree"]), format!("{expected_tree}\n").as_bytes());
    assert_eq!(raw(&f.local(), &["diff", "--name-only"]), b"sub/keep\n");
    assert!(raw(&f.local(), &["diff", "--cached", "--name-only"]).is_empty());
    assert_eq!(f.local().rev_parse("HEAD").unwrap(), Some(new.clone()));
    assert_eq!(f.git.read_ref(ZONE).unwrap(), Some(r.zone.clone()));
    assert!(!r.state.exists()); assert!(!f.lock_path().exists()); assert_one_rebase(f, &r.old, &new);
    // First no-pause repeat can snapshot the now-present upstream file, but never repeats the resolved rebase.
    let again = f.report(&["--json", "resolve"], "resolve", 0, "nothing-to-resolve");
    assert_eq!(again.text("next"), "mr join"); assert_eq!(f.head(), new); assert_one_rebase(f, &r.old, &new);
    let before = f.observe(); f.report(&["--json", "resolve"], "resolve", 0, "nothing-to-resolve");
    assert_eq!(f.observe(), before); assert!(!f.lock_path().exists());
}

#[test]
fn later_commit_repauses_then_both_rounds_resolutions_reach_the_final_folder() {
    let f = Fixture::new(); let first = f.save_topic(); let base = f.base.clone();
    let old = f.commit(&first, &[("sub/keep", b"request second\n")], "second request change\n");
    set_request(&f, &old);
    let zone = f.commit(&base, &[("src/file", b"zone work\n"), ("sub/keep", b"zone second\n"), ("src/upstream", b"advanced zone work\n")], "zone moved\n");
    set_zone(&f, &zone); let r = Paused::start(f, &base, &first); r.first(); let f = &r.f;
    f.write("src/file", b"first chosen resolution\n");
    let report = f.report(&["--json", "resolve"], "resolve", 2, "conflicted");
    assert_eq!(report.text("commit"), old.as_str()); assert_eq!(report.strings("files"), &["sub/keep"]);
    assert_eq!(report.text("next"), "mr resolve"); assert_eq!(report.text("state"), r.state.to_str().unwrap());
    r.assert_unmoved(); assert!(rebase_records(f).is_empty());
    let paused = join::paused_join(&f.local().git_dir(&f.request).unwrap()).unwrap().unwrap();
    assert_eq!(paused.commit, old); assert_eq!(paused.files, vec![std::path::PathBuf::from("sub/keep")]);
    let written = r.first(); assert_eq!(written.strings("files"), &["sub/keep"]);
    assert!(String::from_utf8(fs::read(f.request.join("sub/keep")).unwrap()).unwrap().contains("<<<<<<<"));
    assert_eq!(fs::read(f.request.join("src/file")).unwrap(), b"first chosen resolution\n");
    f.write("sub/keep", b"second chosen resolution\n");
    let expected = tree(f, &zone, &[("src/file", b"first chosen resolution\n"), ("sub/keep", b"second chosen resolution\n")], &[], &[]);
    let done = f.report(&["--json", "resolve"], "resolve", 0, "rebased"); let new = f.head();
    assert_eq!(done.text("from"), old.as_str()); assert_eq!(done.text("to"), new.as_str());
    let info = f.git.commit_info(&new).unwrap(); assert_eq!(info.tree, expected);
    assert_eq!(info.message, "second request change\n");
    let first_info = f.git.commit_info(&info.parents[0]).unwrap(); assert_eq!(first_info.parents, vec![zone.clone()]);
    assert_eq!(first_info.message, f.git.commit_info(&first).unwrap().message);
    assert_eq!(raw(&f.local(), &["write-tree"]), format!("{expected}\n").as_bytes());
    assert!(raw(&f.local(), &["status", "--porcelain"]).is_empty());
    assert_eq!(fs::read(f.request.join("src/file")).unwrap(), b"first chosen resolution\n");
    assert_eq!(fs::read(f.request.join("sub/keep")).unwrap(), b"second chosen resolution\n");
    assert_eq!(fs::read(f.request.join("src/upstream")).unwrap(), b"advanced zone work\n");
    assert_eq!(f.git.read_ref(ZONE).unwrap(), Some(zone)); assert!(!r.state.exists());
    assert_one_rebase(f, &old, &new); assert!(!f.lock_path().exists());
}
