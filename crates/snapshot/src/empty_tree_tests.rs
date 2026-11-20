use crate::empty_tree::empty_tree;
use gitio::Oid;

#[test]
fn a_sha1_id_gives_the_sha1_empty_tree() {
    let like = Oid::parse("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa").unwrap();
    assert_eq!(empty_tree(&like).unwrap().as_str(), "4b825dc642cb6eb9a060e54bf8d69288fbee4904");
}

#[test]
fn a_sha256_id_gives_the_sha256_empty_tree() {
    let like = Oid::parse("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb").unwrap();
    assert_eq!(
        empty_tree(&like).unwrap().as_str(),
        "6ef19b41225c5369f1c104d45d8d85efa9b057b53b14b4b9b939dd74decc5321"
    );
}
