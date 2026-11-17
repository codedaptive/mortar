use super::worktrees_missing;

const LIST: &str = "worktree /tmp/m/work\nHEAD 0a1b\nbranch refs/heads/main\n\n\
                    worktree /tmp/m/zone-a\nHEAD 0a1b\nbranch refs/heads/zone/a\n\n\
                    worktree /tmp/m/r1\nHEAD 4d5e\nbranch refs/heads/req/a/r1\n";

fn folders(paths: &[&str]) -> Vec<String> {
    paths.iter().map(|path| path.to_string()).collect()
}

#[test]
fn listed_folders_are_not_missing() {
    assert!(worktrees_missing(LIST, &folders(&["/tmp/m/zone-a", "/tmp/m/r1"])).is_empty());
}

#[test]
fn an_unlisted_folder_is_missing() {
    assert_eq!(
        worktrees_missing(LIST, &folders(&["/tmp/m/r2", "/tmp/m/r1", "/tmp/m/zone-b"])),
        vec!["/tmp/m/r2".to_string(), "/tmp/m/zone-b".to_string()]
    );
}

#[test]
fn a_path_that_only_starts_the_same_is_missing() {
    assert_eq!(worktrees_missing(LIST, &folders(&["/tmp/m/r"])), vec!["/tmp/m/r".to_string()]);
}
