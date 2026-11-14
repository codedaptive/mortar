use super::run;
use gitio::{Git, Oid};

pub fn blob(git: &Git, tree: &Oid, path: &str) -> Vec<u8> {
    run(git, &["show", &format!("{tree}:{path}")])
}
