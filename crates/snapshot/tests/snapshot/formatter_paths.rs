use crate::support::{assert_snapshot_written, blob, formatter_script, run, tree_names, write, Fixture};
use gitio::{Git, RunOpts};
use snapshot::{snapshot_formatted, FormatterResult, RefusedFile, DEFAULT_SIZE_CAP};
use std::{fs, os::unix::fs::symlink, path::PathBuf};
use testkit::TestRepo;

#[test]
fn formatter_gets_only_regular_saved_nonconflicted_changes_not_deleted_refused_excluded_links_or_submodules() {
    // Joins real status/unmerged index/gitlink -> capped tree eligibility -> child argv -> formatted snapshot/oplog.
    let f = Fixture::new(&[("deleted.txt", b"delete me\n"), ("conflicted.txt", b"base conflict\n")]);
    // The submodule is cloned locally from another disposable repo; no network or real-team repository is used.
    let source = TestRepo::new().unwrap();
    let source_git = Git { repo: source.path.clone(), ..f.git.clone() };
    write(&source.path, "subfile", b"submodule base\n");
    run(&source_git, &["add", "subfile"]);
    run(&source_git, &["commit", "-m", "submodule base"]);
    run(&f.git, &["-c", "protocol.file.allow=always", "submodule", "add", source.path.to_str().unwrap(), "module"]);
    run(&f.git, &["add", "-A"]);
    run(&f.git, &["commit", "-m", "fixture with local submodule"]);
    let head = f.git.rev_parse("HEAD").unwrap().unwrap();
    let module = f.repo.path.join("module");
    let module_git = Git { repo: module.clone(), ..f.git.clone() };
    write(&module, "subfile", b"submodule changed\n");
    run(&module_git, &["add", "subfile"]);
    run(&module_git, &["commit", "-m", "submodule change"]);
    let module_tip = module_git.rev_parse("HEAD").unwrap().unwrap();
    write(&f.repo.path, "root.txt", b"format tracked\n");
    write(&f.repo.path, "eligible new.txt", b"format untracked\n");
    fs::remove_file(f.repo.path.join("deleted.txt")).unwrap();
    write(&f.repo.path, "target/output", b"do not format build output\n");
    write(&f.repo.path, "private/secret", b"do not format caller exclude\n");
    let large = vec![b'x'; 6 * 1024 * 1024];
    write(&f.repo.path, "refused.bin", &large);
    symlink("root.txt", f.repo.path.join("link.txt")).unwrap();
    make_unmerged(&f.git);
    let conflict = b"<<<<<<< ours\nours\n=======\ntheirs\n>>>>>>> theirs\n";
    write(&f.repo.path, "conflicted.txt", conflict);
    let status = run(&f.git, &["status", "--porcelain"]);
    assert!(std::str::from_utf8(&status).unwrap().contains("UU conflicted.txt"));
    let unmerged = run(&f.git, &["ls-files", "--unmerged", "-z"]);
    assert!(!unmerged.is_empty());
    let index_before = fs::read(f.common.join("index")).unwrap();
    let (formatter, trace) = formatter_script(&f, b"for path in \"$@\"; do printf 'formatted\\n' >\"$path\"; done\n");
    let saved = snapshot_formatted(&f.git, &f.repo.path, &f.target, &["private/".into()], DEFAULT_SIZE_CAP, Some(&formatter)).unwrap();
    assert_snapshot_written(&f, &saved, &head);
    let formatted = saved.formatted.as_ref().unwrap();
    assert_eq!(formatted.result, FormatterResult::Formatted);
    assert_eq!(formatted.paths, vec![PathBuf::from("root.txt"), PathBuf::from("eligible new.txt")]);
    assert_eq!(fs::read(&trace).unwrap(), b"literal $(touch unexpected)\0--\0root.txt\0eligible new.txt\0");
    assert_eq!(fs::read(trace.with_extension("count")).unwrap(), b"run\n");
    assert_eq!(tree_names(&f.git, &saved.tree), [".gitmodules", "conflicted.txt", "eligible new.txt", "link.txt", "module", "root.txt"]);
    for path in ["root.txt", "eligible new.txt"] {
        assert_eq!(blob(&f.git, &saved.tree, path), b"formatted\n");
        assert_eq!(fs::read(f.repo.path.join(path)).unwrap(), b"formatted\n");
    }
    assert_eq!(blob(&f.git, &saved.tree, "conflicted.txt"), conflict);
    assert_eq!(fs::read(f.repo.path.join("conflicted.txt")).unwrap(), conflict);
    assert_eq!(fs::read(f.repo.path.join("target/output")).unwrap(), b"do not format build output\n");
    assert_eq!(fs::read(f.repo.path.join("private/secret")).unwrap(), b"do not format caller exclude\n");
    assert_eq!(fs::read(f.repo.path.join("refused.bin")).unwrap(), large);
    assert!(!f.repo.path.join("deleted.txt").exists());
    assert_eq!(fs::read_link(f.repo.path.join("link.txt")).unwrap(), PathBuf::from("root.txt"));
    assert_eq!(blob(&f.git, &saved.tree, "link.txt"), b"root.txt");
    assert_eq!(fs::read(module.join("subfile")).unwrap(), b"submodule changed\n");
    assert_eq!(run(&f.git, &["ls-tree", saved.tree.as_str(), "module"]), format!("160000 commit {module_tip}\tmodule\n").as_bytes());
    assert_eq!(saved.refused, vec![RefusedFile { path: PathBuf::from("refused.bin"), size: 6 * 1024 * 1024, cap: 5 * 1024 * 1024 }]);
    assert_eq!(fs::read(f.common.join("index")).unwrap(), index_before);
    assert_eq!(run(&f.git, &["ls-files", "--unmerged", "-z"]), unmerged);
}

fn make_unmerged(git: &Git) {
    // Real stage 1/2/3 entries, not a fake StatusPaths; no merge/replay implementation is under test here.
    let mut entries = String::new();
    for (stage, content) in [(1, b"base conflict\n".as_slice()), (2, b"ours\n".as_slice()), (3, b"theirs\n".as_slice())] {
        let output = git.run(&["hash-object", "-w", "--stdin"], &RunOpts { stdin: Some(content.to_vec()), ..RunOpts::default() }).unwrap();
        let oid = std::str::from_utf8(&output.stdout).unwrap().trim_end();
        entries.push_str(&format!("100644 {oid} {stage}\tconflicted.txt\n"));
    }
    run(git, &["update-index", "--force-remove", "conflicted.txt"]);
    git.run(&["update-index", "--index-info"], &RunOpts { stdin: Some(entries.into_bytes()), ..RunOpts::default() }).unwrap();
}
