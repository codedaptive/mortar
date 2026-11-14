//! extra_ref_verdict: refs/heads/, refs/mortar/ and refs/remotes/ names are refused even when absent; a protected
//! ref is refused naming its protector; an absent ref is skipped; a present one is accepted with its id.

use gitio::Oid;
use oplog::ProtectedRef;

use super::extra_ref_verdict::extra_ref_verdict;
use super::ExtraRef;

const A: &str = "0123456789abcdef0123456789abcdef01234567";
const B: &str = "fedcba9876543210fedcba9876543210fedcba98";

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn refs(names: &[(&str, &str)]) -> Vec<(Oid, String)> {
    names.iter().map(|(hex, name)| (oid(hex), name.to_string())).collect()
}

fn guard(refname: &str, by: &str) -> ProtectedRef {
    ProtectedRef {
        refname: refname.to_string(),
        by: by.to_string(),
    }
}

#[test]
fn a_branch_is_refused() {
    let input = refs(&[(A, "refs/heads/main")]);
    assert_eq!(
        extra_ref_verdict("refs/heads/main", &input, &[]),
        ExtraRef::Refuse("is under refs/heads/".to_string())
    );
}

#[test]
fn an_absent_branch_is_refused_not_skipped() {
    assert_eq!(
        extra_ref_verdict("refs/heads/gone", &[], &[]),
        ExtraRef::Refuse("is under refs/heads/".to_string())
    );
}

#[test]
fn a_mortar_ref_is_refused() {
    let input = refs(&[(A, "refs/mortar/snap/z/r")]);
    assert_eq!(
        extra_ref_verdict("refs/mortar/snap/z/r", &input, &[]),
        ExtraRef::Refuse("is under refs/mortar/".to_string())
    );
}

#[test]
fn a_remote_ref_is_refused() {
    let input = refs(&[(A, "refs/remotes/origin/main")]);
    assert_eq!(
        extra_ref_verdict("refs/remotes/origin/main", &input, &[]),
        ExtraRef::Refuse("is under refs/remotes/".to_string())
    );
}

#[test]
fn a_protected_ref_is_refused_naming_its_protector() {
    let input = refs(&[(A, "refs/proven/z")]);
    let protected = [guard("refs/t3/other", "forge"), guard("refs/proven/z", "land")];
    assert_eq!(
        extra_ref_verdict("refs/proven/z", &input, &protected),
        ExtraRef::Refuse("is protected by land".to_string())
    );
}

#[test]
fn an_absent_ref_is_skipped() {
    let input = refs(&[(A, "refs/t3/checkpoint/other")]);
    assert_eq!(extra_ref_verdict("refs/t3/checkpoint/1", &input, &[]), ExtraRef::Skip);
}

#[test]
fn a_present_ref_is_accepted_with_its_id() {
    let input = refs(&[(A, "refs/t3/checkpoint/0"), (B, "refs/t3/checkpoint/1")]);
    assert_eq!(extra_ref_verdict("refs/t3/checkpoint/1", &input, &[]), ExtraRef::Accept(oid(B)));
}

#[test]
fn a_prefix_needs_its_slash() {
    let input = refs(&[(B, "refs/headsx")]);
    assert_eq!(extra_ref_verdict("refs/headsx", &input, &[]), ExtraRef::Accept(oid(B)));
}
