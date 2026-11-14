use crate::push_request::push_request;
use crate::{Oid, PushLease, PushRef};
use std::ffi::OsString;

const A: &str = "fb863af4e7dd149507b87a188f4b02b2e0093eb2";
const B: &str = "587f694cd3d0783cac8389a0e2f6beae2f15ce32";

fn push_ref(refname: &str, new: Option<&str>, lease: PushLease) -> PushRef {
    PushRef {
        refname: refname.into(),
        new: new.map(|hex| Oid::parse(hex).unwrap()),
        lease,
    }
}

fn os(args: &[&str]) -> Vec<OsString> {
    args.iter().map(OsString::from).collect()
}

#[test]
fn unleased_refs_give_one_refspec_each_after_the_remote() {
    let refs = [
        push_ref("refs/heads/main", Some(A), PushLease::Unleased),
        push_ref("refs/heads/side", Some(B), PushLease::Unleased),
    ];
    let (args, _) = push_request("../remote.git", &refs).unwrap();
    let want = os(&[
        "push",
        "--porcelain",
        "--no-follow-tags",
        "--end-of-options",
        "../remote.git",
        "fb863af4e7dd149507b87a188f4b02b2e0093eb2:refs/heads/main",
        "587f694cd3d0783cac8389a0e2f6beae2f15ce32:refs/heads/side",
    ]);
    assert_eq!(args, want);
}

#[test]
fn leases_come_in_ref_order_before_end_of_options_and_a_delete_is_colon_refname() {
    let refs = [
        push_ref("refs/heads/main", Some(B), PushLease::At(Oid::parse(A).unwrap())),
        push_ref("refs/heads/free", Some(A), PushLease::Unleased),
        push_ref("refs/heads/new", Some(A), PushLease::Absent),
        push_ref("refs/heads/gone", None, PushLease::At(Oid::parse(B).unwrap())),
    ];
    let (args, _) = push_request("origin", &refs).unwrap();
    let want = os(&[
        "push",
        "--porcelain",
        "--no-follow-tags",
        "--force-with-lease=refs/heads/main:fb863af4e7dd149507b87a188f4b02b2e0093eb2",
        "--force-with-lease=refs/heads/new:",
        "--force-with-lease=refs/heads/gone:587f694cd3d0783cac8389a0e2f6beae2f15ce32",
        "--end-of-options",
        "origin",
        "587f694cd3d0783cac8389a0e2f6beae2f15ce32:refs/heads/main",
        "fb863af4e7dd149507b87a188f4b02b2e0093eb2:refs/heads/free",
        "fb863af4e7dd149507b87a188f4b02b2e0093eb2:refs/heads/new",
        ":refs/heads/gone",
    ]);
    assert_eq!(args, want);
}

#[test]
fn opts_are_a_write_with_exit_1_an_answer_and_no_env_of_their_own() {
    let (_, opts) = push_request("origin", &[push_ref("refs/heads/main", Some(A), PushLease::Unleased)]).unwrap();
    assert_eq!(opts.answer_codes, vec![1]);
    assert!(!opts.read_only);
    assert!(opts.env.is_empty());
    assert_eq!(opts.stdin, None);
    assert_eq!(opts.cwd, None);
}

#[test]
fn no_refs_is_refused() {
    let err = push_request("origin", &[]).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("at least one ref"), "{}", err.stderr);
}

#[test]
fn a_ref_named_twice_is_refused_naming_it() {
    let refs = [
        push_ref("refs/heads/a", Some(A), PushLease::Unleased),
        push_ref("refs/heads/b", Some(A), PushLease::Unleased),
        push_ref("refs/heads/a", None, PushLease::Absent),
    ];
    let err = push_request("origin", &refs).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.stderr.contains("twice"), "{}", err.stderr);
    assert!(err.stderr.contains("\"refs/heads/a\""), "{}", err.stderr);
}
