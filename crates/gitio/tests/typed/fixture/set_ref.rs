use gitio::{Git, Oid, RunOpts};

pub fn set_ref(git: &Git, name: &str, oid: &Oid) {
    git.run(&["update-ref", name, oid.as_str()], &RunOpts::default()).unwrap();
}
