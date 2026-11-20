use crate::snapshot_parents::snapshot_parents;
use gitio::Oid;

fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

#[test]
fn first_snapshot_has_head_alone() {
    let head = oid("1111111111111111111111111111111111111111");
    assert_eq!(snapshot_parents(&head, None), vec![oid("1111111111111111111111111111111111111111")]);
}

#[test]
fn later_snapshot_has_head_then_previous() {
    let head = oid("1111111111111111111111111111111111111111");
    let previous = oid("2222222222222222222222222222222222222222");
    assert_eq!(
        snapshot_parents(&head, Some(&previous)),
        vec![oid("1111111111111111111111111111111111111111"), oid("2222222222222222222222222222222222222222")]
    );
}
