use super::run;
use gitio::{Git, Oid};

pub fn tree_names(git: &Git, tree: &Oid) -> Vec<String> {
    run(git, &["ls-tree", "-r", "--name-only", "-z", tree.as_str()])
        .split(|b| *b == 0).filter(|s| !s.is_empty())
        .map(|s| std::str::from_utf8(s).unwrap().to_owned()).collect()
}
