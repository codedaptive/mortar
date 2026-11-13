use super::{set_ref, Fixture};
use crate::repository::TestRepo;
use gitio::{Oid, RunOpts};

impl Fixture {
    pub fn new() -> Self {
        let repo = TestRepo::new();
        // Non-branch refs may point at blobs: no author identity, clocks, or working index needed.
        let objects = ["old\n", "new\n", "foreign\n"].map(|content| {
            let output = repo.git.run(&["hash-object", "-w", "--stdin"], &RunOpts {
                stdin: Some(content.as_bytes().to_vec()),
                ..RunOpts::default()
            }).unwrap();
            Oid::parse(String::from_utf8(output.stdout).unwrap().trim_end()).unwrap()
        });
        for name in ["refs/mortar/a", "refs/mortar/b"] {
            set_ref(&repo.git, name, &objects[0]);
        }
        let common = repo.path.join(".git");
        Self { repo, common, objects }
    }
}
