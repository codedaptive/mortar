use crate::support::TestRepo;
use gitio::RunOpts;
use std::fs;

#[test]
fn git_run_applies_git_environment_then_per_run_overrides() {
    // Joins Git::new -> Git::run with real git's GIT_DIR/GIT_WORK_TREE selection.
    let first = TestRepo::new();
    let second = TestRepo::new();
    let mut git = first.git.clone();
    git.env.extend([
        ("GIT_DIR".into(), second.path.join(".git").into_os_string()),
        ("GIT_WORK_TREE".into(), second.path.as_os_str().to_owned()),
    ]);
    let from_git = git.run(&["rev-parse", "--show-toplevel"], &RunOpts::default()).unwrap();
    let from_run = git.run(&["rev-parse", "--show-toplevel"], &RunOpts {
        env: vec![
            ("GIT_DIR".into(), first.path.join(".git").into_os_string()),
            ("GIT_WORK_TREE".into(), first.path.as_os_str().to_owned()),
        ],
        ..RunOpts::default()
    }).unwrap();
    for (output, expected) in [(from_git, &second.path), (from_run, &first.path)] {
        let path = String::from_utf8(output.stdout).unwrap();
        assert_eq!(std::path::Path::new(path.trim_end()), expected.canonicalize().unwrap());
    }
}

#[test]
fn git_run_explicit_index_and_object_store_do_not_touch_defaults() {
    // Joins Git::run's env and stdin paths with git add, ls-files, hash-object and cat-file.
    let repo = TestRepo::new();
    fs::write(repo.path.join("tracked"), b"hello\n").unwrap();
    let alternate_index = repo.path.join("alternate-index");
    let index_opts = RunOpts {
        env: vec![("GIT_INDEX_FILE".into(), alternate_index.as_os_str().to_owned())],
        ..RunOpts::default()
    };
    repo.git.run(&["add", "--", "tracked"], &index_opts).unwrap();
    assert!(alternate_index.is_file());
    assert!(!repo.path.join(".git/index").exists());
    let indexed = repo.git.run(&["ls-files", "-z"], &index_opts).unwrap();
    assert_eq!(indexed.stdout, b"tracked\0");
    let normal = repo.git.run(&["ls-files", "-z"], &RunOpts::default()).unwrap();
    assert!(normal.stdout.is_empty());

    // Use different bytes from git add, so the default object store cannot already have the blob.
    let objects = repo.path.join("alternate-objects");
    fs::create_dir(&objects).unwrap();
    let object_opts = RunOpts {
        env: vec![("GIT_OBJECT_DIRECTORY".into(), objects.as_os_str().to_owned())],
        stdin: Some(b"private object\0\xff\n".to_vec()),
        ..RunOpts::default()
    };
    let written = repo.git.run(&["hash-object", "-w", "--stdin"], &object_opts).unwrap();
    let oid = String::from_utf8(written.stdout).unwrap();
    let read_opts = RunOpts { stdin: None, ..object_opts };
    let blob = repo.git.run(&["cat-file", "blob", oid.trim_end()], &read_opts).unwrap();
    assert_eq!(blob.stdout, b"private object\0\xff\n");
    let missing = repo.git.run(&["cat-file", "blob", oid.trim_end()], &RunOpts::default()).unwrap_err();
    assert_eq!(missing.code, Some(128));
    assert!(missing.stderr.contains("fatal:"), "{missing:?}");
}
