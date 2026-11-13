use crate::update_refs_stdin::update_refs_stdin;
use crate::{Oid, RefUpdate};

const A1: &str = "1111111111111111111111111111111111111111";
const B1: &str = "2222222222222222222222222222222222222222";
const C1: &str = "3333333333333333333333333333333333333333";
const A256: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn update(refname: &str, new: &str, old: Option<&str>) -> RefUpdate {
    RefUpdate {
        refname: refname.to_string(),
        new: Oid::parse(new).unwrap(),
        old: old.map(|hex| Oid::parse(hex).unwrap()),
    }
}

#[test]
fn one_update_with_old_given() {
    let bytes = update_refs_stdin(&[update("refs/heads/x", A1, Some(B1))]).unwrap();
    assert_eq!(
        bytes,
        b"start\0update refs/heads/x\x001111111111111111111111111111111111111111\x002222222222222222222222222222222222222222\0prepare\0commit\0"
            .to_vec()
    );
}

#[test]
fn absent_old_is_40_zeros_for_sha1() {
    let bytes = update_refs_stdin(&[update("refs/heads/x", A1, None)]).unwrap();
    assert_eq!(
        bytes,
        b"start\0update refs/heads/x\x001111111111111111111111111111111111111111\x000000000000000000000000000000000000000000\0prepare\0commit\0"
            .to_vec()
    );
}

#[test]
fn absent_old_is_64_zeros_for_sha256() {
    let bytes = update_refs_stdin(&[update("refs/heads/x", A256, None)]).unwrap();
    assert_eq!(
        bytes,
        b"start\0update refs/heads/x\0aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\x000000000000000000000000000000000000000000000000000000000000000000\0prepare\0commit\0"
            .to_vec()
    );
}

#[test]
fn three_updates_keep_their_order() {
    let bytes = update_refs_stdin(&[
        update("refs/heads/c", C1, Some(A1)),
        update("refs/heads/a", A1, None),
        update("refs/tags/b", B1, Some(C1)),
    ])
    .unwrap();
    assert_eq!(
        bytes,
        b"start\0\
update refs/heads/c\x003333333333333333333333333333333333333333\x001111111111111111111111111111111111111111\0\
update refs/heads/a\x001111111111111111111111111111111111111111\x000000000000000000000000000000000000000000\0\
update refs/tags/b\x002222222222222222222222222222222222222222\x003333333333333333333333333333333333333333\0\
prepare\0commit\0"
            .to_vec()
    );
}

#[test]
fn no_updates_is_start_prepare_commit() {
    assert_eq!(update_refs_stdin(&[]).unwrap(), b"start\0prepare\0commit\0".to_vec());
}

#[test]
fn empty_refname_is_refused() {
    let err = update_refs_stdin(&[update("", A1, None)]).unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("\"\""), "{}", err.stderr);
}

#[test]
fn nul_in_refname_is_refused_naming_it() {
    let err = update_refs_stdin(&[
        update("refs/heads/ok", A1, None),
        update("refs/heads/x\0update refs/heads/y", B1, None),
    ])
    .unwrap_err();
    assert_eq!(err.code, None);
    assert!(err.args.is_empty());
    assert!(err.stderr.contains("\"refs/heads/x\\0update refs/heads/y\""), "{}", err.stderr);
}
