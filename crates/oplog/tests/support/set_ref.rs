use gitio::{Git, Oid, RunOpts};

/// Seed/observe independently of oplog::apply and Git::update_refs.
pub fn set_ref(git: &Git, name: &str, value: &Oid) {
    git.run(&["update-ref", name, value.as_str()], &RunOpts::default()).unwrap();
}
