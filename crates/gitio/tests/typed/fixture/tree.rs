use gitio::{Git, Oid, RunOpts};

/// Build an oracle tree through raw git and a fixture-only index, never a typed operation.
pub fn tree(git: &Git, files: &[(&str, &[u8])]) -> Oid {
    let opts = RunOpts {
        env: vec![("GIT_INDEX_FILE".into(), git.repo.join(".git/typed-fixture-index").into_os_string())],
        ..RunOpts::default()
    };
    git.run(&["read-tree", "--empty"], &opts).unwrap();
    for (path, contents) in files {
        let written = git.run(&["hash-object", "-w", "--stdin"], &RunOpts {
            stdin: Some(contents.to_vec()), ..RunOpts::default()
        }).unwrap();
        let oid = String::from_utf8(written.stdout).unwrap();
        git.run(&["update-index", "--add", "--cacheinfo", "100644", oid.trim_end(), path], &opts).unwrap();
    }
    let written = git.run(&["write-tree"], &opts).unwrap();
    Oid::parse(String::from_utf8(written.stdout).unwrap().trim_end()).unwrap()
}
