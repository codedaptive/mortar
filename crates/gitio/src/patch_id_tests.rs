use crate::PatchId;
use std::collections::HashSet;

const A: &str = "0000000000000000000000000000000000000001";
const B: &str = "00000000000000000000000000000000000000ff";

#[test]
fn equal_ids_hash_as_one() {
    let set: HashSet<PatchId> = [A, A, B].iter().map(|s| PatchId::parse(s).unwrap()).collect();
    assert_eq!(set.len(), 2);
}

#[test]
fn orders_by_hex_text() {
    let mut ids = [PatchId::parse(B).unwrap(), PatchId::parse(A).unwrap()];
    ids.sort();
    assert_eq!(ids[0].as_str(), "0000000000000000000000000000000000000001");
    assert_eq!(ids[1].as_str(), "00000000000000000000000000000000000000ff");
}
