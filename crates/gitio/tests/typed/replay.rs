use crate::fixture::{blob, commit, repo, set_ref, tree};
use gitio::{Oid, ReplayOutcome, RunOpts};

#[test]
fn replay_probe_gates_real_clean_conflict_and_error_results_without_ref_moves() {
    // S-12: joins replay_available -> real replay -> parsed proposed updates, never a ref move.
    let r = repo();
    let base_tree = tree(&r.git, &[("file", b"base\n")]);
    let base = commit(&r.git, &base_tree, &[], "base\n");
    let topic_tree = tree(&r.git, &[("file", b"topic\n")]);
    let topic = commit(&r.git, &topic_tree, &[base.clone()], "topic change\n");
    let onto_tree = tree(&r.git, &[("file", b"base\n"), ("upstream", b"upstream\n")]);
    let onto = commit(&r.git, &onto_tree, &[base.clone()], "upstream change\n");
    set_ref(&r.git, "refs/heads/main", &base);
    set_ref(&r.git, "refs/heads/topic", &topic);
    let before = r.git.run(&["for-each-ref", "--format=%(refname) %(objectname)"], &RunOpts::default()).unwrap().stdout;
    let help = match r.git.run(&["replay", "-h"], &RunOpts {
        answer_codes: vec![1, 129], ..RunOpts::default()
    }) {
        Ok(output) => [output.stdout, output.stderr].concat(),
        Err(error) => error.stderr.into_bytes(),
    };
    let supported = String::from_utf8_lossy(&help).contains("--ref-action");
    assert_eq!(r.git.replay_available().unwrap(), supported);
    if !supported {
        assert!(r.git.replay(&onto, &base, "refs/heads/topic").is_err());
        let after = r.git.run(&["for-each-ref", "--format=%(refname) %(objectname)"], &RunOpts::default()).unwrap().stdout;
        assert_eq!(after, before, "an unsupported optional replay must not move refs");
        return;
    }

    let updates = match r.git.replay(&onto, &base, "refs/heads/topic").unwrap() {
        ReplayOutcome::Replayed(updates) => updates,
        other => panic!("expected clean replay: {other:?}"),
    };
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].refname, "refs/heads/topic");
    assert_eq!(updates[0].old, Some(topic.clone()));
    assert_ne!(updates[0].new, topic);
    let expected_tree = tree(&r.git, &[("file", b"topic\n"), ("upstream", b"upstream\n")]);
    let commit_shape = r.git.run(&["show", "-s", "--format=%T%n%P", updates[0].new.as_str()], &RunOpts::default()).unwrap();
    assert_eq!(commit_shape.stdout, format!("{expected_tree}\n{onto}\n").as_bytes());
    assert_eq!(blob(&r.git, &updates[0].new, "file"), b"topic\n");
    assert_eq!(blob(&r.git, &updates[0].new, "upstream"), b"upstream\n");

    let conflict_tree = tree(&r.git, &[("file", b"onto conflict\n")]);
    let conflict = commit(&r.git, &conflict_tree, &[base.clone()], "conflicting upstream\n");
    assert_eq!(r.git.replay(&conflict, &base, "refs/heads/topic").unwrap(), ReplayOutcome::Conflicted);
    let missing = Oid::parse(&"0".repeat(40)).unwrap();
    assert!(r.git.replay(&onto, &missing, "refs/heads/topic").is_err());
    let after = r.git.run(&["for-each-ref", "--format=%(refname) %(objectname)"], &RunOpts::default()).unwrap().stdout;
    assert_eq!(after, before, "clean/conflicted/failed replay must all leave refs alone");
}
