use gitio::{Git, Oid, RunOpts};

/// Read tree contents through raw git rather than another typed operation under test.
pub fn blob(git: &Git, tree: &Oid, path: &str) -> Vec<u8> {
    git.run(&["show", &format!("{tree}:{path}")], &RunOpts {
        read_only: true, ..RunOpts::default()
    }).unwrap().stdout
}
