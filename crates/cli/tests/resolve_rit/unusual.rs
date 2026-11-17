use crate::{objects::*, paused::Paused, support::*};
use gitio::Git;
use std::fs;

#[test]
fn rit_real_rename_rename_lists_absent_original_as_skipped_then_keeps_only_the_chosen_name() {
    let f = Fixture::new(); let base = f.base.clone();
    let zone_tree = tree(&f, &base, &[("src/ours", b"base\n"), ("src/upstream", b"advanced zone work\n")], &["src/file"], &[]);
    let zone = commit(&f, &base, &zone_tree, "zone renames\n");
    let request_tree = tree(&f, &base, &[("src/theirs", b"base\n")], &["src/file"], &[]);
    let old = commit(&f, &base, &request_tree, "request renames\n");
    set_zone(&f, &zone); set_request(&f, &old); let r = Paused::start(f, &base, &old); let f = &r.f;
    let first = r.first(); assert_eq!(first.strings("skipped"), &["src/file"]);
    assert_eq!(first.strings("files"), &["src/ours", "src/theirs"]); assert!(first.strings("gitlinks").is_empty());
    assert!(first.lines().contains("skipped (no version in the conflicted tree): src/file"));
    assert!(!f.request.join("src/file").exists()); r.assert_unmoved();
    f.write("src/ours", b"chosen rename bytes\n"); fs::remove_file(f.request.join("src/theirs")).unwrap();
    let expected = tree(f, &zone, &[("src/ours", b"chosen rename bytes\n")], &[], &[]);
    f.report(&["--json", "resolve"], "resolve", 0, "rebased"); let new = f.head();
    assert_eq!(f.git.commit_info(&new).unwrap().tree, expected);
    assert_eq!(fs::read(f.request.join("src/ours")).unwrap(), b"chosen rename bytes\n");
    assert!(!f.request.join("src/file").exists()); assert!(!f.request.join("src/theirs").exists());
    assert_eq!(fs::read(f.request.join("src/upstream")).unwrap(), b"advanced zone work\n");
    assert_eq!(raw(&f.local(), &["write-tree"]), format!("{expected}\n").as_bytes());
    assert!(raw(&f.local(), &["status", "--porcelain"]).is_empty()); assert_one_rebase(f, &old, &new);
    assert!(!r.state.exists()); assert!(!f.lock_path().exists());
}

#[test]
fn rit_real_gitlink_is_listed_without_writes_then_actual_child_repo_head_selects_the_resolution() {
    let f = Fixture::new(); let empty = oid(&raw(&f.git, &["mktree"]));
    let sub_base = f.git.commit_tree(&empty, &[], "sub base\n", None, None).unwrap();
    let sub_zone = f.git.commit_tree(&empty, std::slice::from_ref(&sub_base), "sub zone\n", None, None).unwrap();
    let sub_request = f.git.commit_tree(&empty, std::slice::from_ref(&sub_base), "sub request\n", None, None).unwrap();
    let base_tree = tree(&f, &f.base, &[(".gitmodules", b"[submodule \"module\"]\n\tpath = src/module\n\turl = ./unused\n")], &[], &[("src/module", &sub_base)]);
    let base = commit(&f, &f.base, &base_tree, "base with submodule\n");
    let zone_tree = tree(&f, &base, &[("src/upstream", b"advanced zone work\n")], &[], &[("src/module", &sub_zone)]);
    let zone = commit(&f, &base, &zone_tree, "zone submodule\n");
    let request_tree = tree(&f, &base, &[], &[], &[("src/module", &sub_request)]);
    let old = commit(&f, &base, &request_tree, "request submodule\n");
    set_zone(&f, &zone); set_request(&f, &old);
    let module = f.request.join("src/module");
    raw(&f.git, &["init", "-q", module.to_str().unwrap()]);
    // The child is a real repo with its own HEAD/index; its objects borrow only this disposable fixture's store.
    fs::write(module.join(".git/objects/info/alternates"), format!("{}\n", f.common.join("objects").display())).unwrap();
    let sub = Git { repo: module.clone(), ..f.git.clone() };
    raw(&sub, &["checkout", "--detach", sub_request.as_str()]);
    let r = Paused::start(f, &base, &old); let f = &r.f;
    write(&module, "local-marker", b"<<<<<<< nested submodule work must survive\n");
    let first = r.first(); assert!(first.strings("files").is_empty()); assert!(first.strings("skipped").is_empty());
    assert_eq!(first.strings("gitlinks"), &["src/module"]);
    assert!(first.lines().contains("submodule src/module: set its commit in the folder, then mr resolve"));
    assert_eq!(fs::read(module.join("local-marker")).unwrap(), b"<<<<<<< nested submodule work must survive\n");
    assert_eq!(sub.rev_parse("HEAD").unwrap(), Some(sub_request.clone())); r.assert_unmoved();
    let chosen = f.git.commit_tree(&empty, &[sub_zone, sub_request], "chosen submodule merge\n", None, None).unwrap();
    raw(&sub, &["checkout", "--detach", chosen.as_str()]);
    f.report(&["--json", "resolve"], "resolve", 0, "rebased"); let new = f.head();
    assert_eq!(raw(&f.git, &["ls-tree", new.as_str(), "--", "src/module"]), format!("160000 commit {chosen}\tsrc/module\n").as_bytes());
    let snap = f.git.read_ref(SNAP).unwrap().unwrap();
    assert_eq!(raw(&f.git, &["ls-tree", snap.as_str(), "--", "src/module"]), format!("160000 commit {chosen}\tsrc/module\n").as_bytes());
    assert_eq!(sub.rev_parse("HEAD").unwrap(), Some(chosen));
    assert_eq!(fs::read(module.join("local-marker")).unwrap(), b"<<<<<<< nested submodule work must survive\n");
    assert_eq!(fs::read(f.request.join("src/upstream")).unwrap(), b"advanced zone work\n");
    assert_eq!(raw(&f.local(), &["write-tree"]), format!("{}\n", f.git.commit_info(&new).unwrap().tree).as_bytes());
    assert_one_rebase(f, &old, &new); assert!(!r.state.exists()); assert!(!f.lock_path().exists());
}
