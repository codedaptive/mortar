//! Git::push to a local bare remote made inside the test repository: new, up to date, forced under a matching lease,
//! stale lease, Absent lease on an existing ref, non-fast-forward unleased, delete, remote hook refusals (results in
//! request order), a local pre-push refusal, and MORTAR_OP from Git.env seen by the remote's hook. Each push that moved
//! or refused a ref is checked against the remote's refs read with plain git.
use crate::fixture::{commit, repo, tree};
use crate::support::GitRepo;
use gitio::{Git, Oid, PushLease, PushOutcome, PushRef, PushRefusal, RunOpts};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// The local repository, a Git on the bare remote, the remote's path as push's remote argument, and three commits:
/// `first`, `next` on top of it, and `other`, a sibling of `next`, so moving next to other is not a fast-forward.
struct Setup {
    local: GitRepo,
    bare: Git,
    remote: String,
    first: Oid,
    next: Oid,
    other: Oid,
}

fn setup() -> Setup {
    let local = repo();
    let path = local.path.join("remote.git");
    local.git.run(&["init", "-q", "--bare", path.to_str().unwrap()], &RunOpts::default()).unwrap();
    let first = commit(&local.git, &tree(&local.git, &[("a", b"1\n")]), &[], "first\n");
    let next = commit(&local.git, &tree(&local.git, &[("a", b"2\n")]), std::slice::from_ref(&first), "next\n");
    let other = commit(&local.git, &tree(&local.git, &[("a", b"3\n")]), std::slice::from_ref(&first), "other\n");
    let bare = Git { repo: path.clone(), ..local.git.clone() };
    Setup { remote: path.to_str().unwrap().to_string(), local, bare, first, next, other }
}

/// The remote's value of `refname` read with plain git; None when rev-parse answers exit 1 (no such ref).
fn on_remote(bare: &Git, refname: &str) -> Option<Oid> {
    let opts = RunOpts { answer_codes: vec![1], read_only: true, ..RunOpts::default() };
    let out = bare.run(&["rev-parse", "--verify", "--quiet", refname], &opts).unwrap();
    (out.status == 0).then(|| Oid::parse(String::from_utf8(out.stdout).unwrap().trim_end()).unwrap())
}

fn to(refname: &str, new: Option<&Oid>, lease: PushLease) -> PushRef {
    PushRef { refname: refname.into(), new: new.cloned(), lease }
}

fn hook(dir: &Path, name: &str, body: &str) {
    fs::create_dir_all(dir).unwrap();
    let path = dir.join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
}

/// The local Git with MORTAR_OP=x1 added to its env, as oplog's op_git adds it for an op's ref moves.
fn with_op(git: &Git) -> Git {
    let mut git = git.clone();
    git.env.push(("MORTAR_OP".into(), "x1".into()));
    git
}

#[test]
fn push_creates_answers_up_to_date_and_forces_under_a_matching_lease() {
    let s = setup();
    let new = [to("refs/heads/main", Some(&s.first), PushLease::Unleased), to("refs/heads/side", Some(&s.next), PushLease::Unleased)];
    assert_eq!(s.local.git.push(&s.remote, &new).unwrap(), vec![PushOutcome::Pushed, PushOutcome::Pushed]);
    assert_eq!(on_remote(&s.bare, "refs/heads/main"), Some(s.first.clone()));
    assert_eq!(on_remote(&s.bare, "refs/heads/side"), Some(s.next.clone()));
    let same = [to("refs/heads/main", Some(&s.first), PushLease::Unleased)];
    assert_eq!(s.local.git.push(&s.remote, &same).unwrap(), vec![PushOutcome::UpToDate]);
    let forced = [to("refs/heads/side", Some(&s.other), PushLease::At(s.next.clone()))];
    assert_eq!(s.local.git.push(&s.remote, &forced).unwrap(), vec![PushOutcome::Pushed]);
    assert_eq!(on_remote(&s.bare, "refs/heads/side"), Some(s.other.clone()));
}

#[test]
fn push_refuses_a_stale_lease_an_absent_lease_and_a_non_fast_forward_moving_nothing() {
    let s = setup();
    let seed = [to("refs/heads/main", Some(&s.next), PushLease::Unleased)];
    assert_eq!(s.local.git.push(&s.remote, &seed).unwrap(), vec![PushOutcome::Pushed]);
    let cases = [
        (PushLease::At(s.first.clone()), PushRefusal::StaleLease),
        (PushLease::Absent, PushRefusal::StaleLease),
        (PushLease::Unleased, PushRefusal::NonFastForward),
    ];
    for (lease, why) in cases {
        let moved = [to("refs/heads/main", Some(&s.other), lease.clone())];
        assert_eq!(s.local.git.push(&s.remote, &moved).unwrap(), vec![PushOutcome::Refused(why)], "{lease:?}");
        assert_eq!(on_remote(&s.bare, "refs/heads/main"), Some(s.next.clone()), "{lease:?}");
    }
}

#[test]
fn push_deletes_and_refuses_a_delete_under_a_stale_lease() {
    let s = setup();
    let seed = [to("refs/heads/main", Some(&s.first), PushLease::Unleased), to("refs/heads/side", Some(&s.next), PushLease::Unleased)];
    s.local.git.push(&s.remote, &seed).unwrap();
    let stale = [to("refs/heads/side", None, PushLease::At(s.first.clone()))];
    assert_eq!(s.local.git.push(&s.remote, &stale).unwrap(), vec![PushOutcome::Refused(PushRefusal::StaleLease)]);
    assert_eq!(on_remote(&s.bare, "refs/heads/side"), Some(s.next.clone()));
    let delete = [to("refs/heads/main", None, PushLease::Unleased)];
    assert_eq!(s.local.git.push(&s.remote, &delete).unwrap(), vec![PushOutcome::Pushed]);
    assert_eq!(on_remote(&s.bare, "refs/heads/main"), None);
}

#[test]
fn push_answers_remote_hook_refusals_per_ref_in_request_order_and_the_hook_sees_mortar_op() {
    let s = setup();
    let seen = s.local.path.join("op-seen");
    let hooks = s.bare.repo.join("hooks");
    let pre_receive = format!(
        "printf '%s\\n' \"$MORTAR_OP\" >> '{}'\nwhile read old new ref; do\n  [ \"$ref\" = refs/heads/hooked ] && exit 1\ndone\nexit 0\n",
        seen.display()
    );
    hook(&hooks, "pre-receive", &pre_receive);
    hook(&hooks, "update", "[ \"$1\" = refs/heads/blocked ] && exit 1\nexit 0\n");
    let git = with_op(&s.local.git);
    let hooked = [to("refs/heads/hooked", Some(&s.first), PushLease::Unleased)];
    assert_eq!(git.push(&s.remote, &hooked).unwrap(), vec![PushOutcome::Refused(PushRefusal::Hook)]);
    assert_eq!(on_remote(&s.bare, "refs/heads/hooked"), None);
    assert_eq!(fs::read_to_string(&seen).unwrap(), "x1\n");
    let two = [to("refs/heads/blocked", Some(&s.first), PushLease::Unleased), to("refs/heads/ok2", Some(&s.first), PushLease::Unleased)];
    assert_eq!(git.push(&s.remote, &two).unwrap(), vec![PushOutcome::Refused(PushRefusal::Hook), PushOutcome::Pushed]);
    assert_eq!(on_remote(&s.bare, "refs/heads/blocked"), None);
    assert_eq!(on_remote(&s.bare, "refs/heads/ok2"), Some(s.first.clone()));
}

#[test]
fn push_returns_a_local_pre_push_refusal_as_an_error_with_the_hooks_stderr() {
    let s = setup();
    hook(&s.local.path.join(".git/hooks"), "pre-push", "echo \"guard says no op=$MORTAR_OP\" >&2\nexit 1\n");
    let main = [to("refs/heads/main", Some(&s.first), PushLease::Unleased)];
    let err = with_op(&s.local.git).push(&s.remote, &main).unwrap_err();
    assert!(err.stderr.contains("guard says no op=x1\n"), "{}", err.stderr);
    assert_eq!(on_remote(&s.bare, "refs/heads/main"), None);
}
