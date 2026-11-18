//! Production creators reject planted temporary links.

use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::{symlink, PermissionsExt};
use super::super::{apply_step, temp_path, write_record, InstallStep};
use super::support::{snapshot, Tree};

#[test]
fn record_creator_does_not_follow_planted_link() {
    let tree = Tree::new();
    let target = tree.root.join("record");
    let victim = tree.root.join("victim");
    fs::write(&victim, b"victim bytes").unwrap();
    let planted = temp_path(&target, 1);
    symlink(&victim, &planted).unwrap();
    let mut tokens = [1, 2].into_iter();
    write_record(&target, "new record", &mut || tokens.next().unwrap()).unwrap();
    assert_eq!(fs::read(&target).unwrap(), b"new record");
    assert_eq!(fs::read(&victim).unwrap(), b"victim bytes");
    assert_eq!(fs::read_link(&planted).unwrap(), victim);
}

#[test]
fn skill_creator_does_not_follow_planted_link() {
    let tree = Tree::new();
    let target = tree.root.join("skill");
    let victim = tree.root.join("victim");
    fs::write(&victim, b"victim bytes").unwrap();
    let planted = temp_path(&target, 1);
    symlink(&victim, &planted).unwrap();
    let mut tokens = [1, 2].into_iter();
    apply_step(&InstallStep::WriteSkill { path: target.clone(), text: "new skill".into() }, &mut || tokens.next().unwrap()).unwrap();
    assert_eq!(fs::read(&target).unwrap(), b"new skill");
    assert_eq!(fs::read(&victim).unwrap(), b"victim bytes");
    assert_eq!(fs::read_link(&planted).unwrap(), victim);
}

#[test]
fn binary_creator_does_not_follow_planted_link() {
    let tree = Tree::new();
    let target = tree.root.join("binary");
    let victim = tree.root.join("victim");
    let source = tree.root.join("source-binary");
    fs::write(&source, b"binary bytes").unwrap();
    fs::write(&victim, b"victim bytes").unwrap();
    let planted = temp_path(&target, 1);
    symlink(&victim, &planted).unwrap();
    let mut tokens = [1, 2].into_iter();
    apply_step(&InstallStep::PlaceBinary { from: source, to: target.clone() }, &mut || tokens.next().unwrap()).unwrap();
    assert_eq!(fs::read(&target).unwrap(), b"binary bytes");
    assert_eq!(fs::metadata(&target).unwrap().permissions().mode() & 0o777, 0o755);
    assert_eq!(fs::read(&victim).unwrap(), b"victim bytes");
    assert_eq!(fs::read_link(&planted).unwrap(), victim);
}

#[test]
fn link_creator_does_not_replace_planted_link() {
    let tree = Tree::new();
    let target = tree.root.join("link");
    let victim = tree.root.join("victim");
    let destination = tree.root.join("destination");
    fs::write(&victim, b"victim bytes").unwrap();
    let planted = temp_path(&target, 1);
    symlink(&victim, &planted).unwrap();
    let mut tokens = [1, 2].into_iter();
    apply_step(&InstallStep::Link { at: target.clone(), target: destination.clone() }, &mut || tokens.next().unwrap()).unwrap();
    assert_eq!(fs::read_link(&target).unwrap(), destination);
    assert_eq!(fs::read(&victim).unwrap(), b"victim bytes");
    assert_eq!(fs::read_link(&planted).unwrap(), victim);
}

#[test]
fn record_creator_reports_all_eight_names_taken_without_changes() {
    let tree = Tree::new();
    let target = tree.root.join("record");
    for token in 1..=8 { fs::write(temp_path(&target, token), b"taken").unwrap(); }
    let before = snapshot(&tree.root);
    let mut tokens = 1..=8;
    let error = write_record(&target, "new record", &mut || tokens.next().unwrap()).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::AlreadyExists);
    assert_eq!(snapshot(&tree.root), before);
}
