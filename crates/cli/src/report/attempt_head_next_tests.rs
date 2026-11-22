//! attempt_head_next: an unborn HEAD in an attempt folder names the try branch; any other error, or no attempt
//! branch, gives None. Errors are built as data, as join/snapshot_failure_tests does.

use std::path::PathBuf;

use snapshot::SnapshotError;

use super::attempt_head_next;

#[test]
fn attempt_head_next_names_the_try_branch_only_for_an_unborn_head_in_an_attempt() {
    let unborn = || SnapshotError::UnbornHead { folder: PathBuf::from("/w/try-1") };
    let rows: Vec<(&str, SnapshotError, Option<&str>, Option<&str>)> = vec![
        (
            "unborn head in attempt 1",
            unborn(),
            Some("try/core/task/1"),
            Some("check out the branch try/core/task/1 in the attempt's folder, then repeat the same command"),
        ),
        ("unborn head outside an attempt", unborn(), None, None),
        (
            "detached head in an attempt",
            SnapshotError::DetachedHead { folder: PathBuf::from("/w/try-1") },
            Some("try/core/task/1"),
            None,
        ),
        (
            "bad target in an attempt",
            SnapshotError::BadTarget { reason: "attempt 0".to_string() },
            Some("try/core/task/1"),
            None,
        ),
    ];
    let failures: Vec<String> = rows
        .into_iter()
        .filter_map(|(case, error, branch, expected)| {
            let got = attempt_head_next(&error, branch);
            (got.as_deref() != expected).then(|| format!("{case}: expected {expected:?}, got {got:?}"))
        })
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
