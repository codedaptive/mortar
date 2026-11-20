//! Real commit_info records and the empty-date passthrough.
use crate::fixture::{commit, repo, tree};
use gitio::{Oid, Person, RunOpts};

#[test]
fn commit_info_root_one_parent_and_merge_keep_exact_tree_message_author_and_git_parent_order() {
    // Joins raw git commit creation -> Git::commit_info -> independently specified trees/parents/author/messages.
    let r = repo();
    let root_tree = tree(&r.git, &[("file", b"root\n")]);
    let root = commit(&r.git, &root_tree, &[], "root subject\n\nroot body\n");
    let one_tree = tree(&r.git, &[("file", b"one\n")]);
    let one = commit(&r.git, &one_tree, std::slice::from_ref(&root), "one-parent subject\n");
    let side_tree = tree(&r.git, &[("file", b"root\n"), ("side", b"side\n")]);
    let side = commit(&r.git, &side_tree, std::slice::from_ref(&root), "side\n");
    let merge_tree = tree(&r.git, &[("file", b"one\n"), ("side", b"side\n")]);
    let merge = commit(&r.git, &merge_tree, &[side.clone(), one.clone()], "merge subject\n\nbody\n\n");
    for (id, contents, parents, message) in [
        (&root, &root_tree, vec![], "root subject\n\nroot body\n"),
        (&one, &one_tree, vec![root.clone()], "one-parent subject\n"),
        (&merge, &merge_tree, vec![side.clone(), one.clone()], "merge subject\n\nbody\n\n"),
    ] {
        let info = r.git.commit_info(id).unwrap();
        assert_eq!(&info.tree, contents);
        assert_eq!(info.parents, parents);
        assert_eq!(info.message, message);
        assert_eq!(info.author, Person {
            name: "Fixture Author".into(), email: "author@example.invalid".into(), date: Some("946684800 +0000".into()),
        });
        // Raw git independently prints parents in object order; the literal fixture order above also has to match.
        let listed = r.git.run(&["show", "-s", "--format=%P", id.as_str()], &RunOpts::default()).unwrap().stdout;
        let expected = format!("{}\n", parents.iter().map(Oid::as_str).collect::<Vec<_>>().join(" "));
        assert_eq!(listed, expected.as_bytes());
    }
}

#[test]
fn commit_info_keeps_an_empty_author_date_and_the_rest_of_the_real_commit_readable() {
    // Joins an actual malformed commit -> real git output -> metadata passthrough, without repairing its date.
    // Snapshot/save need its tree and parents; the refusal belongs to join replay, not commit_info.
    let r = repo();
    let contents = tree(&r.git, &[("file", b"preserved\n")]);
    let raw = format!("tree {contents}\nauthor No Date <empty-date@example.invalid> \ncommitter Fixture Committer <committer@example.invalid> 946771200 +0000\n\nempty author date\n");
    let written = r.git.run(&["hash-object", "--literally", "-t", "commit", "-w", "--stdin"], &RunOpts {
        stdin: Some(raw.as_bytes().to_vec()), ..RunOpts::default()
    }).unwrap();
    let malformed = Oid::parse(std::str::from_utf8(&written.stdout).unwrap().trim_end()).unwrap();
    assert_eq!(r.git.run(&["cat-file", "commit", malformed.as_str()], &RunOpts::default()).unwrap().stdout, raw.as_bytes());
    let info = r.git.commit_info(&malformed).unwrap();
    assert_eq!(info.tree, contents);
    assert!(info.parents.is_empty());
    assert_eq!(info.author, Person {
        name: "No Date".into(), email: "empty-date@example.invalid".into(), date: Some(String::new()),
    });
    assert_eq!(info.message, "empty author date\n");
}
