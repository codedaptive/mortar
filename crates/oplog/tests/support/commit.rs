use gitio::{Git, Oid, RunOpts};

/// Builds a real commit with the exact parents supplied by the test, without touching refs or the index.
pub fn commit(git: &Git, label: &str, parents: &[&Oid]) -> Oid {
    let tree = git.run(&["mktree"], &RunOpts {
        stdin: Some(Vec::new()), ..RunOpts::default()
    }).unwrap();
    let tree = String::from_utf8(tree.stdout).unwrap();
    let mut args = vec!["commit-tree", tree.trim(), "-m", label];
    for parent in parents {
        args.extend(["-p", parent.as_str()]);
    }
    let out = git.run(&args, &RunOpts {
        env: [
            ("GIT_AUTHOR_NAME", "Integration"),
            ("GIT_AUTHOR_EMAIL", "integration@example.invalid"),
            ("GIT_COMMITTER_NAME", "Integration"),
            ("GIT_COMMITTER_EMAIL", "integration@example.invalid"),
            ("GIT_AUTHOR_DATE", "2000-01-01T00:00:00+00:00"),
            ("GIT_COMMITTER_DATE", "2000-01-01T00:00:00+00:00"),
        ].map(|(key, value)| (key.into(), value.into())).to_vec(),
        ..RunOpts::default()
    }).unwrap();
    Oid::parse(String::from_utf8(out.stdout).unwrap().trim()).unwrap()
}
