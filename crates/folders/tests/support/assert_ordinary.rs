use super::run;
use gitio::{Git, Oid};
use std::{fs, path::Path};

pub fn assert_ordinary(git: &Git, folder: &Path, branch: &str, head: &Oid, cone: &[&str]) {
    let local = Git { repo: folder.to_owned(), ..git.clone() };
    let list = run(git, &["worktree", "list", "--porcelain"]);
    let list = std::str::from_utf8(&list).unwrap();
    let block = format!("worktree {}\nHEAD {head}\nbranch refs/heads/{branch}\n", fs::canonicalize(folder).unwrap().display());
    assert!(list.contains(&block), "{list}");
    assert_eq!(run(&local, &["symbolic-ref", "HEAD"]), format!("refs/heads/{branch}\n").as_bytes());
    assert_eq!(run(&local, &["status", "--porcelain"]), b"");
    assert_eq!(run(&local, &["config", "--get", "core.sparseCheckoutCone"]), b"true\n");
    assert_eq!(run(&local, &["sparse-checkout", "list"]), format!("{}\n", cone.join("\n")).as_bytes());
    assert_eq!(fs::read(folder.join("Cargo.toml")).unwrap(), b"# root build file\n");
    assert_eq!(fs::read(folder.join("README")).unwrap(), b"root file\n");
    assert!(!folder.join("other").exists());
}
