//! name_moves: one move per name, in order, each with old None and new 40 zeros.

use super::name_moves;

#[test]
fn names_are_kept_in_order() {
    let refs = vec!["refs/heads/b".to_string(), String::new(), "refs/heads/a".to_string()];
    let names: Vec<String> = name_moves(&refs)
        .unwrap()
        .into_iter()
        .map(|m| m.refname)
        .collect();
    assert_eq!(names, ["refs/heads/b", "", "refs/heads/a"]);
}

#[test]
fn old_is_none() {
    let moves = name_moves(&["refs/heads/main".to_string()]).unwrap();
    assert_eq!(moves[0].old, None);
}

#[test]
fn new_is_forty_zeros() {
    let moves = name_moves(&["refs/heads/main".to_string()]).unwrap();
    assert_eq!(moves[0].new.as_str(), "0000000000000000000000000000000000000000");
}

#[test]
fn no_names_give_no_moves() {
    assert!(name_moves(&[]).unwrap().is_empty());
}
