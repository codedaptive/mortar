use crate::fixture::{commit, repo, set_ref, tree};
use gitio::{Git, RunOpts};
use std::fs;

#[test]
fn worktree_lifecycle_sparse_paths_and_discovery_share_the_real_common_store() {
    // Joins worktree_add/list/remove -> sparse_set -> common_dir/toplevel on real linked worktrees.
    let r = repo();
    let contents = tree(&r.git, &[("root.txt", b"root\n"), ("keep/deep/file", b"included\n"), ("outside/file", b"excluded\n")]);
    let base = commit(&r.git, &contents, &[], "base\n");
    set_ref(&r.git, "refs/heads/main", &base);
    let linked = r.path.join("linked folder");
    let detached = r.path.join("detached");
    let vanished = r.path.join("vanished");
    r.git.worktree_add(&linked, &base, Some("topic")).unwrap();
    r.git.worktree_add(&detached, &base, None).unwrap();
    r.git.worktree_add(&vanished, &base, None).unwrap();
    for folder in [&linked, &detached, &vanished] {
        assert!(folder.join(".git").is_file());
        assert_eq!(fs::read_dir(folder).unwrap().count(), 1, "--no-checkout must leave only .git");
    }
    r.git.sparse_set(&linked, &["keep".into()]).unwrap();
    assert_eq!(fs::read(linked.join("root.txt")).unwrap(), b"root\n");
    assert_eq!(fs::read(linked.join("keep/deep/file")).unwrap(), b"included\n");
    assert!(!linked.join("outside").exists());
    let nested = Git { repo: linked.join("keep/deep"), ..r.git.clone() };
    assert_eq!(nested.toplevel().unwrap(), linked.canonicalize().unwrap());
    assert_eq!(nested.common_dir().unwrap(), r.path.join(".git").canonicalize().unwrap());
    assert_eq!(r.git.toplevel().unwrap(), r.path.canonicalize().unwrap());
    assert_eq!(r.git.common_dir().unwrap(), r.path.join(".git").canonicalize().unwrap());

    r.git.run(&["worktree", "lock", "--reason", "integration test", detached.to_str().unwrap()], &RunOpts::default()).unwrap();
    let vanished_path = vanished.canonicalize().unwrap();
    fs::remove_dir_all(&vanished).unwrap();
    let list = r.git.worktree_list().unwrap();
    assert_eq!(list.len(), 4);
    assert_eq!(list[0].path, r.path.canonicalize().unwrap());
    assert_eq!(list[0].head, Some(base.clone()));
    assert_eq!(list[0].branch.as_deref(), Some("refs/heads/main"));
    assert!(!list[0].bare && !list[0].detached);
    let topic = list.iter().find(|entry| entry.path == linked.canonicalize().unwrap()).unwrap();
    assert_eq!(topic.head, Some(base.clone()));
    assert_eq!(topic.branch.as_deref(), Some("refs/heads/topic"));
    assert!(!topic.detached && !topic.locked && !topic.prunable);
    let locked = list.iter().find(|entry| entry.path == detached.canonicalize().unwrap()).unwrap();
    assert_eq!(locked.head, Some(base));
    assert!(locked.detached && locked.locked);
    assert_eq!(locked.branch, None);
    assert!(list.iter().find(|entry| entry.path == vanished_path).unwrap().prunable);

    fs::write(linked.join("keep/deep/file"), b"modified\n").unwrap();
    assert!(r.git.worktree_remove(&linked, false).is_err());
    assert_eq!(fs::read(linked.join("keep/deep/file")).unwrap(), b"modified\n");
    r.git.worktree_remove(&linked, true).unwrap();
    assert!(!linked.exists());
    assert!(!r.git.worktree_list().unwrap().iter().any(|entry| entry.branch.as_deref() == Some("refs/heads/topic")));
    r.git.run(&["worktree", "unlock", detached.to_str().unwrap()], &RunOpts::default()).unwrap();
    r.git.worktree_remove(&detached, true).unwrap();
    assert!(!detached.exists());
}
