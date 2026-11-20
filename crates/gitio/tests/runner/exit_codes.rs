use crate::support::GitRepo;
use gitio::RunOpts;
use std::fs;

#[test]
fn git_run_preserves_diff_answer_and_rejects_other_codes() {
    // Joins Git::new -> Git::run -> real git diff's exit-code and output protocol.
    let repo = GitRepo::new();
    fs::write(repo.path.join("left"), b"before\n").unwrap();
    fs::write(repo.path.join("right"), b"after\n").unwrap();
    let args = ["diff", "--no-index", "--no-color", "--", "left", "right"];

    let error = repo.git.run(&args, &RunOpts::default()).unwrap_err();
    assert_eq!(error.code, Some(1));
    assert_eq!(error.args, args);

    let opts = RunOpts {
        answer_codes: vec![1],
        read_only: true,
        ..RunOpts::default()
    };
    let answer = repo.git.run(&args, &opts).unwrap();
    assert_eq!(answer.status, 1);
    let diff = String::from_utf8(answer.stdout).unwrap();
    assert!(diff.contains("-before\n+after\n"), "{diff}");
    assert!(answer.stderr.is_empty());

    let error = repo.git.run(&["rev-parse", "--verify", "refs/heads/missing"], &opts).unwrap_err();
    assert_eq!(error.code, Some(128));
    assert_eq!(error.args, ["rev-parse", "--verify", "refs/heads/missing"]);
    assert!(error.stderr.contains("fatal:"), "{error:?}");
}
