use gitio::{Git, RunOpts};

/// Raw git observations/fixture setup, independent of snapshot's decisions.
pub fn run(git: &Git, args: &[&str]) -> Vec<u8> {
    git.run(args, &RunOpts::default()).unwrap().stdout
}
