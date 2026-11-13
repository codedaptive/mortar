use crate::support::TestRepo;
use gitio::RunOpts;
use std::fs;

#[test]
fn git_run_round_trips_stdin_and_passes_literal_arguments() {
    // Joins Git::run's argument/stdin construction with git hash-object and cat-file.
    let repo = TestRepo::new();
    let output = repo.git.run(&["hash-object", "-w", "--stdin"], &RunOpts {
        stdin: Some(b"hello\n".to_vec()),
        ..RunOpts::default()
    }).unwrap();
    assert_eq!(output.status, 0);
    assert_eq!(output.stdout, b"ce013625030ba8dba906f756967f9e9ca394464a\n");
    let content = repo.git.run(
        &["cat-file", "blob", "ce013625030ba8dba906f756967f9e9ca394464a"],
        &RunOpts::default(),
    ).unwrap();
    assert_eq!(content.stdout, b"hello\n");

    let literal = "literal;$(touch MUST_NOT_EXIST)";
    fs::write(repo.path.join(literal), b"hello\n").unwrap();
    let output = repo.git.run(&["hash-object", "--", literal], &RunOpts::default()).unwrap();
    assert_eq!(output.stdout, b"ce013625030ba8dba906f756967f9e9ca394464a\n");
    assert!(!repo.path.join("MUST_NOT_EXIST").exists());
}

#[test]
fn git_run_cwd_override_selects_another_real_repository() {
    // Joins Git::new -> Git::run with git's -C repository selection.
    let first = TestRepo::new();
    let second = TestRepo::new();
    for (cwd, expected) in [(None, &first.path), (Some(second.path.clone()), &second.path)] {
        let output = first.git.run(&["rev-parse", "--show-toplevel"], &RunOpts {
            cwd,
            read_only: true,
            ..RunOpts::default()
        }).unwrap();
        let actual = String::from_utf8(output.stdout).unwrap();
        assert_eq!(std::path::Path::new(actual.trim_end()), expected.canonicalize().unwrap());
    }
}

#[test]
fn test_repo_removes_its_repository_on_drop() {
    // Joins TestRepo::new -> Git::run (git init) -> TestRepo::drop.
    let path = {
        let repo = TestRepo::new();
        assert!(repo.path.join(".git/HEAD").is_file());
        repo.path.clone()
    };
    assert!(!path.exists(), "fixture left {} behind", path.display());
}
