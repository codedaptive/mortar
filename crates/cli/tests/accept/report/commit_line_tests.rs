use super::commit_line;

const ID: &str = "0123456789abcdef0123456789abcdef01234567";

#[test]
fn the_commit_and_whether_the_tree_differs_from_it() {
    let rows = [
        (Ok(ID.to_string()), Ok(false), "commit 0123456789abcdef0123456789abcdef01234567"),
        (Ok(ID.to_string()), Ok(true), "commit 0123456789abcdef0123456789abcdef01234567 with local changes"),
        (
            Ok(ID.to_string()),
            Err("git status --porcelain exited 128: not a repo".to_string()),
            "commit 0123456789abcdef0123456789abcdef01234567 (whether it has local changes was not read: git status \
             --porcelain exited 128: not a repo)",
        ),
        (
            Err("git rev-parse HEAD exited 128: no HEAD".to_string()),
            Ok(true),
            "commit not read: git rev-parse HEAD exited 128: no HEAD",
        ),
        (
            Err("git rev-parse HEAD did not start: no git".to_string()),
            Err("git status --porcelain did not start: no git".to_string()),
            "commit not read: git rev-parse HEAD did not start: no git",
        ),
    ];
    for (head, changed, expected) in rows {
        assert_eq!(commit_line(&head, &changed), expected, "head {head:?}, changed {changed:?}");
    }
}
