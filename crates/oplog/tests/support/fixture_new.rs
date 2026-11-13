use super::{set_ref, Fixture};
use gitio::{Git, Oid, RunOpts};
use testkit::TestRepo;

impl Fixture {
    pub fn new() -> Self {
        let repo = TestRepo::new().unwrap();
        let mut git = Git::new(&repo.path);
        git.env = vec![
            ("GIT_CONFIG_NOSYSTEM".into(), "1".into()),
            ("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()),
        ];
        // Non-branch refs may point at blobs: no author identity, clocks, or working index needed.
        let objects = ["old\n", "new\n", "foreign\n"].map(|content| {
            let output = git.run(&["hash-object", "-w", "--stdin"], &RunOpts {
                stdin: Some(content.as_bytes().to_vec()),
                ..RunOpts::default()
            }).unwrap();
            Oid::parse(String::from_utf8(output.stdout).unwrap().trim_end()).unwrap()
        });
        for name in ["refs/mortar/a", "refs/mortar/b"] {
            set_ref(&git, name, &objects[0]);
        }
        let common = repo.path.join(".git");
        Self { repo, git, common, objects }
    }
}
