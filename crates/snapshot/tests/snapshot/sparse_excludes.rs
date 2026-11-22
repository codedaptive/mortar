use crate::support::{blob, run, tree_names, write, Fixture};
use gitio::RunOpts;
use snapshot::{saved_state, snapshot, DEFAULT_SIZE_CAP};
use std::{fs, path::{Path, PathBuf}};

#[test]
fn sparse_snapshot_keeps_out_of_cone_tree_and_preserves_staged_index_and_all_files() {
    // Joins snapshot -> gitio worktree_add/sparse_set/snapshot_tree -> oplog ref/log in the common store.
    let f = Fixture::new(&[("keep/edited", b"base kept\n"), ("outside/hidden", b"hidden original\n")]);
    let folder = f.repo.path.join("sparse-folder");
    f.git.worktree_add(&folder, &f.head, None).unwrap();
    f.git.sparse_set(&folder, &["keep".into()]).unwrap();
    let git = gitio::Git { repo: folder.clone(), ..f.git.clone() };
    assert!(!folder.join("outside").exists());
    write(&folder, "keep/edited", b"staged version\n");
    run(&git, &["add", "keep/edited"]);
    write(&folder, "keep/edited", b"unstaged version\n");
    write(&folder, "keep/new", b"untracked version\n");
    let index_path = run(&git, &["rev-parse", "--path-format=absolute", "--git-path", "index"]);
    let index_path = PathBuf::from(std::str::from_utf8(&index_path).unwrap().trim_end());
    let index_before = fs::read(&index_path).unwrap();
    let stage_before = run(&git, &["ls-files", "--stage"]);
    let files_before = files(&folder);
    // Deliberately pass the main repo's Git: snapshot must rebind it to the sparse folder.
    let saved = snapshot(&f.git, &folder, &f.target, &[], DEFAULT_SIZE_CAP).unwrap();
    assert!(saved.written);
    assert_eq!(tree_names(&git, &saved.tree), ["keep/edited", "keep/new", "outside/hidden", "root.txt"]);
    assert_eq!(blob(&git, &saved.tree, "keep/edited"), b"unstaged version\n");
    assert_eq!(blob(&git, &saved.tree, "keep/new"), b"untracked version\n");
    assert_eq!(blob(&git, &saved.tree, "outside/hidden"), b"hidden original\n");
    assert_eq!(blob(&git, &saved.tree, "root.txt"), b"base\n");
    assert_eq!(fs::read(&index_path).unwrap(), index_before);
    assert_eq!(run(&git, &["ls-files", "--stage"]), stage_before);
    assert_eq!(files(&folder), files_before);
    assert!(!folder.join("outside").exists());
    assert_eq!(git.rev_parse("HEAD").unwrap(), Some(f.head.clone()));
    assert_eq!(git.read_ref(&f.refname).unwrap(), Some(saved.commit.clone()));
    assert_eq!(f.git.read_ref(&f.refname).unwrap(), Some(saved.commit));
    assert_eq!(oplog::read_op_log(&f.common).unwrap().records.len(), 2);
    assert!(saved_state(&f.git, &folder, &f.target, &[], DEFAULT_SIZE_CAP).unwrap().saved);
    assert_eq!(fs::read(&index_path).unwrap(), index_before);
    assert_eq!(files(&folder), files_before);
}

#[test]
fn ignored_build_outputs_and_caller_excludes_stay_out_but_similar_names_are_saved() {
    // Joins snapshot_excludes (via snapshot/saved_state) -> gitio's real pathspec/gitignore tree -> oplog.
    let f = Fixture::new(&[(".gitignore", b"ignored/\n*.ignored\n")]);
    let excluded = [
        "ignored/secret", "file.ignored", "target/root", "pkg/target/nested",
        "node_modules/root", "pkg/node_modules/nested", ".build/root", "pkg/.build/nested", "private/secret",
    ];
    for name in excluded { write(&f.repo.path, name, b"must stay out\n"); }
    for name in ["targetish/kept", "pkg/source", "node_modules.txt"] {
        write(&f.repo.path, name, b"must be saved\n");
    }
    let files_before = files_except_git(&f.repo.path);
    let index_before = fs::read(f.common.join("index")).unwrap();
    let excludes = ["private/".into()];
    let saved = snapshot(&f.git, &f.repo.path, &f.target, &excludes, DEFAULT_SIZE_CAP).unwrap();
    assert!(saved.written);
    assert!(saved.refused.is_empty());
    assert_eq!(tree_names(&f.git, &saved.tree), [".gitignore", "node_modules.txt", "pkg/source", "root.txt", "targetish/kept"]);
    for name in ["targetish/kept", "pkg/source", "node_modules.txt"] {
        assert_eq!(blob(&f.git, &saved.tree, name), b"must be saved\n");
    }
    for name in excluded {
        assert!(f.git.run(&["show", &format!("{}:{name}", saved.tree)], &RunOpts::default()).is_err());
    }
    assert_eq!(fs::read(f.common.join("index")).unwrap(), index_before);
    assert_eq!(files_except_git(&f.repo.path), files_before);
    let log_before = fs::read(oplog::oplog_path(&f.common)).unwrap();
    for name in excluded { write(&f.repo.path, name, b"still excluded after changing\n"); }
    assert!(
        saved_state(&f.git, &f.repo.path, &f.target, &excludes, DEFAULT_SIZE_CAP)
            .unwrap()
            .saved
    );
    assert_eq!(fs::read(oplog::oplog_path(&f.common)).unwrap(), log_before);
}

fn files(folder: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    collect(folder, folder, false)
}

fn files_except_git(folder: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    collect(folder, folder, true)
}

fn collect(root: &Path, folder: &Path, omit_git: bool) -> Vec<(PathBuf, Vec<u8>)> {
    let mut result = Vec::new();
    for entry in fs::read_dir(folder).unwrap() {
        let path = entry.unwrap().path();
        if omit_git && path == root.join(".git") { continue; }
        if path.is_dir() { result.extend(collect(root, &path, omit_git)); }
        else { result.push((path.strip_prefix(root).unwrap().to_owned(), fs::read(path).unwrap())); }
    }
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}
