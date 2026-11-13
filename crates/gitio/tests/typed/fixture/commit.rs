use gitio::{Git, Oid, RunOpts};

/// Raw git fixture construction, independent of Git::commit_tree.
pub fn commit(git: &Git, tree: &Oid, parents: &[Oid], message: &str) -> Oid {
    let mut args = vec!["commit-tree".to_string(), tree.as_str().to_string()];
    for parent in parents {
        args.extend(["-p".into(), parent.as_str().into()]);
    }
    let output = git.run(&args, &RunOpts {
        stdin: Some(message.as_bytes().to_vec()), ..RunOpts::default()
    }).unwrap();
    Oid::parse(String::from_utf8(output.stdout).unwrap().trim_end()).unwrap()
}
