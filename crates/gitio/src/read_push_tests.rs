use crate::read_push::read_push;
use crate::{Oid, Output, PushLease, PushOutcome, PushRef, PushRefusal};

fn output(status: i32, stdout: &[u8], stderr: &[u8]) -> Output {
    Output {
        status,
        stdout: stdout.to_vec(),
        stderr: stderr.to_vec(),
    }
}

/// Refs as requested; read_push matches by refname only, so every one asks for the same value.
fn refs(names: &[&str]) -> Vec<PushRef> {
    names
        .iter()
        .map(|name| PushRef {
            refname: name.to_string(),
            new: Some(Oid::parse("fb863af4e7dd149507b87a188f4b02b2e0093eb2").unwrap()),
            lease: PushLease::Unleased,
        })
        .collect()
}

fn refused(why: PushRefusal) -> PushOutcome {
    PushOutcome::Refused(why)
}

#[test]
fn new_branches_are_pushed() {
    let out = output(0, b"To ../remote.git\n*\tfb863af4e7dd149507b87a188f4b02b2e0093eb2:refs/heads/main\t[new branch]\n*\t587f694cd3d0783cac8389a0e2f6beae2f15ce32:refs/heads/side\t[new branch]\nDone\n", b"");
    let got = read_push(&out, &refs(&["refs/heads/main", "refs/heads/side"]));
    assert_eq!(got, Ok(vec![PushOutcome::Pushed, PushOutcome::Pushed]));
}

#[test]
fn equals_is_up_to_date() {
    let out = output(0, b"To ../remote.git\n=\tfb863af4e7dd149507b87a188f4b02b2e0093eb2:refs/heads/main\t[up to date]\nDone\n", b"");
    assert_eq!(read_push(&out, &refs(&["refs/heads/main"])), Ok(vec![PushOutcome::UpToDate]));
}

#[test]
fn answers_follow_request_order_not_record_order() {
    let out = output(1, b"To ../remote.git\n \tfb863af4e7dd149507b87a188f4b02b2e0093eb2:refs/heads/side\t587f694..fb863af\n!\t587f694cd3d0783cac8389a0e2f6beae2f15ce32:refs/heads/main\t[rejected] (non-fast-forward)\nDone\n", b"");
    let got = read_push(&out, &refs(&["refs/heads/main", "refs/heads/side"]));
    assert_eq!(got, Ok(vec![refused(PushRefusal::NonFastForward), PushOutcome::Pushed]));
}

#[test]
fn stale_info_is_a_stale_lease_also_for_a_delete() {
    let out = output(1, b"To ../remote.git\n!\t587f694cd3d0783cac8389a0e2f6beae2f15ce32:refs/heads/main\t[rejected] (stale info)\nDone\n", b"");
    assert_eq!(read_push(&out, &refs(&["refs/heads/main"])), Ok(vec![refused(PushRefusal::StaleLease)]));
    let out = output(1, b"To ../remote.git\n!\t(delete):refs/heads/ff\t[rejected] (stale info)\nDone\n", b"");
    assert_eq!(read_push(&out, &refs(&["refs/heads/ff"])), Ok(vec![refused(PushRefusal::StaleLease)]));
}

#[test]
fn forced_and_deleted_are_pushed() {
    let out = output(0, b"To ../remote.git\n+\t587f694cd3d0783cac8389a0e2f6beae2f15ce32:refs/heads/main\tfb863af...587f694 (forced update)\nDone\n", b"");
    assert_eq!(read_push(&out, &refs(&["refs/heads/main"])), Ok(vec![PushOutcome::Pushed]));
    let out = output(0, b"To ../remote.git\n-\t:refs/heads/side\t[deleted]\nDone\n", b"");
    assert_eq!(read_push(&out, &refs(&["refs/heads/side"])), Ok(vec![PushOutcome::Pushed]));
}

#[test]
fn both_hook_summaries_are_hook_refusals() {
    let out = output(1, b"To ../remote.git\n!\tfb863af4e7dd149507b87a188f4b02b2e0093eb2:refs/heads/hooked\t[remote rejected] (pre-receive hook declined)\nDone\n", b"");
    assert_eq!(read_push(&out, &refs(&["refs/heads/hooked"])), Ok(vec![refused(PushRefusal::Hook)]));
    let out = output(1, b"To ../remote.git\n*\tfb863af4e7dd149507b87a188f4b02b2e0093eb2:refs/heads/ok2\t[new branch]\n!\tfb863af4e7dd149507b87a188f4b02b2e0093eb2:refs/heads/blocked\t[remote rejected] (hook declined)\nDone\n", b"");
    let got = read_push(&out, &refs(&["refs/heads/blocked", "refs/heads/ok2"]));
    assert_eq!(got, Ok(vec![refused(PushRefusal::Hook), PushOutcome::Pushed]));
}

#[test]
fn other_summaries_are_kept_verbatim_and_done_may_be_missing() {
    let out = output(1, b"To ../remote.git\n!\tfb863af4e7dd149507b87a188f4b02b2e0093eb2:refs/heads/ff\t[rejected] (fetch first)\nDone\n", b"");
    let want = refused(PushRefusal::Other("[rejected] (fetch first)".into()));
    assert_eq!(read_push(&out, &refs(&["refs/heads/ff"])), Ok(vec![want]));
    let out = output(1, b"To ../rem.git\n!\t1111111111111111111111111111111111111111:refs/heads/q\t[remote rejected] (unpacker error)\n", b"");
    let want = refused(PushRefusal::Other("[remote rejected] (unpacker error)".into()));
    assert_eq!(read_push(&out, &refs(&["refs/heads/q"])), Ok(vec![want]));
}

#[test]
fn exit_1_with_no_records_is_refused_carrying_git_stderr_verbatim() {
    let out = output(1, b"", b"guard says no op=x1\nerror: failed to push some refs to '../rem.git'\n");
    let err = read_push(&out, &refs(&["refs/heads/main"])).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("exit"), "{}", err.stderr);
    assert!(err.stderr.contains("guard says no op=x1\nerror: failed to push some refs to '../rem.git'\n"), "{}", err.stderr);
    let out = output(1, b"", b"error: src refspec refs/heads/nosuchsrc does not match any\nerror: failed to push some refs to '../remote.git'\n");
    let err = read_push(&out, &refs(&["refs/heads/main"])).unwrap_err();
    assert!(err.stderr.contains("error: src refspec refs/heads/nosuchsrc does not match any\n"), "{}", err.stderr);
}

#[test]
fn exit_and_records_that_disagree_are_refused() {
    let rejected = b"To ../remote.git\n!\t587f694cd3d0783cac8389a0e2f6beae2f15ce32:refs/heads/main\t[rejected] (stale info)\nDone\n";
    assert_eq!(read_push(&output(0, rejected, b""), &refs(&["refs/heads/main"])).unwrap_err().code, None);
    let pushed = b"To ../remote.git\n-\t:refs/heads/main\t[deleted]\nDone\n";
    assert_eq!(read_push(&output(1, pushed, b""), &refs(&["refs/heads/main"])).unwrap_err().code, None);
    assert!(read_push(&output(0, b"", b""), &refs(&["refs/heads/main"])).is_err());
}

#[test]
fn records_must_match_the_requested_refs_one_to_one() {
    let one = b"To ../remote.git\n-\t:refs/heads/side\t[deleted]\nDone\n";
    let err = read_push(&output(0, one, b""), &refs(&["refs/heads/main"])).unwrap_err();
    assert!(err.stderr.contains("\"refs/heads/side\""), "{}", err.stderr);
    let err = read_push(&output(0, one, b""), &refs(&["refs/heads/side", "refs/heads/main"])).unwrap_err();
    assert!(err.stderr.contains("\"refs/heads/main\""), "{}", err.stderr);
    let twice = b"-\t:refs/heads/side\t[deleted]\n-\t:refs/heads/side\t[deleted]\n";
    let err = read_push(&output(0, twice, b""), &refs(&["refs/heads/side"])).unwrap_err();
    assert!(err.stderr.contains("\"refs/heads/side\""), "{}", err.stderr);
}

#[test]
fn malformed_lines_are_refused_naming_them() {
    for (stdout, named) in [
        (&b"-\t:refs/heads/side\t[deleted]"[..], "\"-\\t:refs/heads/side\\t[deleted]\""),
        (b"?\tfb863af4e7dd149507b87a188f4b02b2e0093eb2:refs/heads/side\t[new branch]\n", "\"?\\t"),
        (b"*\trefs/heads/side\t[new branch]\n", "\"*\\trefs/heads/side\\t[new branch]\""),
        (b"*\tfb863af4e7dd149507b87a188f4b02b2e0093eb2:\t[new branch]\n", "\"*\\t"),
        (b"*\tfb863af4e7dd149507b87a188f4b02b2e0093eb2:refs/heads/side\n", "\"*\\t"),
    ] {
        let err = read_push(&output(0, stdout, b""), &refs(&["refs/heads/side"])).unwrap_err();
        assert_eq!(err.code, None);
        assert!(err.stderr.contains(named), "{}", err.stderr);
    }
}
