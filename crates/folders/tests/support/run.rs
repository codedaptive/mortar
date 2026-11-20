use gitio::{Git, RunOpts};

pub fn run(git: &Git, args: &[&str]) -> Vec<u8> {
    git.run(args, &RunOpts::default()).unwrap().stdout
}
