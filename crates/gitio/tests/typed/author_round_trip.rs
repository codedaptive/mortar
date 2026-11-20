//! commit_info's stored author is reusable by commit_tree.
use crate::fixture::{commit, repo, tree};
use gitio::{Oid, Person, RunOpts};

#[test]
fn commit_info_author_with_non_ascii_name_and_non_utc_date_rebuilds_the_same_commit_id() {
    // Joins Git::commit_info -> Person -> Git::commit_tree through real git objects.
    // The oracle is an independently assembled raw commit, not commit_tree's own output.
    let r = repo();
    let empty = tree(&r.git, &[]);
    let first = commit(&r.git, &empty, &[], "first parent\n");
    let second = commit(&r.git, &empty, &[], "second parent\n");
    let contents = tree(&r.git, &[("file", b"preserved payload\n")]);
    let message = "author round trip\n\nbody with trailing blank line\n\n";
    let raw = format!(
        "tree {contents}\nparent {second}\nparent {first}\nauthor Zoë Ångström <zoe@example.invalid> 1700000000 -0530\ncommitter Cora Committer <cora@example.invalid> 1700000200 +0930\n\n{message}"
    );
    let written = r.git.run(&["hash-object", "-t", "commit", "-w", "--stdin"], &RunOpts {
        stdin: Some(raw.as_bytes().to_vec()), ..RunOpts::default()
    }).unwrap();
    let original = Oid::parse(std::str::from_utf8(&written.stdout).unwrap().trim_end()).unwrap();
    let info = r.git.commit_info(&original).unwrap();
    assert_eq!(info.author, Person {
        name: "Zoë Ångström".into(),
        email: "zoe@example.invalid".into(),
        date: Some("1700000000 -0530".into()),
    });
    assert_eq!(info.tree, contents);
    assert_eq!(info.parents, vec![second, first]);
    assert_eq!(info.message, message);
    // The unchanged committer is supplied independently; fixture defaults differ in every identity field.
    let committer = Person {
        name: "Cora Committer".into(),
        email: "cora@example.invalid".into(),
        date: Some("1700000200 +0930".into()),
    };
    let rebuilt = r.git.commit_tree(
        &info.tree, &info.parents, &info.message, Some(&info.author), Some(&committer)
    ).unwrap();
    assert_eq!(rebuilt, original, "author bytes and stored date must survive the round trip");
    let observed = r.git.run(&["cat-file", "commit", rebuilt.as_str()], &RunOpts::default()).unwrap();
    assert_eq!(observed.stdout, raw.as_bytes());
}
