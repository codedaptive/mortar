//! zero_oid_like: 40 zeros for a SHA-1 id, 64 for a SHA-256 id.

use gitio::Oid;

use super::zero_oid_like;

#[test]
fn sha1_gives_forty_zeros() {
    let oid = Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
    assert_eq!(zero_oid_like(&oid).unwrap().as_str(), "0000000000000000000000000000000000000000");
}

#[test]
fn sha256_gives_sixty_four_zeros() {
    let oid = Oid::parse("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap();
    assert_eq!(
        zero_oid_like(&oid).unwrap().as_str(),
        "0000000000000000000000000000000000000000000000000000000000000000"
    );
}
