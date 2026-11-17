//! WO-orc-gitio-it-5, S-39 / SPEC-join WO-join-1: replay uses its explicit attribute source, not disk or onto.
use crate::fixture::{blob, commit, repo, set_ref, tree};
use gitio::{ReplayOutcome, RunOpts};
use std::fs;

#[test]
fn replay_union_comes_from_the_supplied_commit_and_plain_source_conflicts_despite_worktree_union() {
    // Joins replay_available -> Git::replay(attr_source, ...) -> real merge driver/tree and proposed ref updates.
    let r = repo();
    let base_tree = tree(&r.git, &[("log.txt", b"base\n")]);
    let base = commit(&r.git, &base_tree, &[], "base\n");
    let topic_tree = tree(&r.git, &[("log.txt", b"topic\n")]);
    let topic = commit(&r.git, &topic_tree, std::slice::from_ref(&base), "topic shared-line edit\n");
    let onto_tree = tree(&r.git, &[("log.txt", b"onto\n")]);
    let onto = commit(&r.git, &onto_tree, std::slice::from_ref(&base), "upstream shared-line edit\n");
    let union_tree = tree(&r.git, &[("log.txt", b"base\n"), (".gitattributes", b"log.txt merge=union\n")]);
    let union_source = commit(&r.git, &union_tree, std::slice::from_ref(&base), "attributes on a separate branch\n");
    assert_ne!(union_source, onto);
    assert_ne!(union_source, topic);
    set_ref(&r.git, "refs/heads/main", &onto);
    set_ref(&r.git, "refs/heads/topic", &topic);
    r.git.run(&["checkout", "--force", "main"], &RunOpts::default()).unwrap();
    // Deliberately disagree with the supplied source: disk explicitly disables textual merging.
    fs::write(r.path.join(".gitattributes"), b"log.txt -merge\n").unwrap();
    fs::write(r.path.join("log.txt"), b"dirty worktree bytes are not replay input\n").unwrap();
    assert_eq!(r.git.run(&["check-attr", "merge", "--", "log.txt"], &RunOpts::default()).unwrap().stdout,
        b"log.txt: merge: unset\n");
    let refs_before = r.git.run(&["for-each-ref", "--format=%(refname) %(objectname)"], &RunOpts::default()).unwrap().stdout;
    let index_before = fs::read(r.path.join(".git/index")).unwrap();
    let help = match r.git.run(&["replay", "-h"], &RunOpts {
        answer_codes: vec![1, 129], ..RunOpts::default()
    }) {
        Ok(output) => [output.stdout, output.stderr].concat(),
        Err(error) => error.stderr.into_bytes(),
    };
    let supported = String::from_utf8_lossy(&help).contains("--ref-action");
    assert_eq!(r.git.replay_available().unwrap(), supported);
    if !supported {
        eprintln!("S-39 capability gate: --ref-action unavailable; only unsupported/no-ref-move outcome exercised");
        assert!(r.git.replay(&union_source, &onto, &base, "refs/heads/topic").is_err());
    } else {
        eprintln!("S-39 capability gate: --ref-action available; exercising real union/plain attribute-source contrast");
        let updates = match r.git.replay(&union_source, &onto, &base, "refs/heads/topic").unwrap() {
            ReplayOutcome::Replayed(updates) => updates,
            other => panic!("supplied union source must resolve shared-line edit, despite binary disk attributes: {other:?}"),
        };
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].refname, "refs/heads/topic");
        assert_eq!(updates[0].old, Some(topic.clone()));
        assert_ne!(updates[0].new, topic);
        let expected_tree = tree(&r.git, &[("log.txt", b"onto\ntopic\n")]);
        let shape = r.git.run(&["show", "-s", "--format=%T%n%P", updates[0].new.as_str()], &RunOpts::default()).unwrap();
        assert_eq!(shape.stdout, format!("{expected_tree}\n{onto}\n").as_bytes());
        assert_eq!(blob(&r.git, &updates[0].new, "log.txt"), b"onto\ntopic\n");
        assert_eq!(r.git.run(&["ls-tree", "-r", "--name-only", "-z", updates[0].new.as_str()], &RunOpts::default()).unwrap().stdout,
            b"log.txt\0", "attribute-source contents must not be injected into the replayed tree");
        assert_eq!(fs::read(r.path.join(".gitattributes")).unwrap(), b"log.txt -merge\n");
        // Now disk enables union, but the supplied base commit has no .gitattributes: the same replay conflicts.
        fs::write(r.path.join(".gitattributes"), b"log.txt merge=union\n").unwrap();
        assert_eq!(r.git.run(&["check-attr", "merge", "--", "log.txt"], &RunOpts::default()).unwrap().stdout,
            b"log.txt: merge: union\n");
        assert_eq!(r.git.replay(&base, &onto, &base, "refs/heads/topic").unwrap(), ReplayOutcome::Conflicted);
        assert_eq!(fs::read(r.path.join(".gitattributes")).unwrap(), b"log.txt merge=union\n");
    }
    assert_eq!(r.git.run(&["for-each-ref", "--format=%(refname) %(objectname)"], &RunOpts::default()).unwrap().stdout, refs_before);
    assert_eq!(r.git.read_ref("refs/heads/topic").unwrap(), Some(topic));
    assert_eq!(r.git.rev_parse("HEAD").unwrap(), Some(onto));
    assert_eq!(fs::read(r.path.join(".git/index")).unwrap(), index_before);
    assert_eq!(fs::read(r.path.join("log.txt")).unwrap(), b"dirty worktree bytes are not replay input\n");
}
