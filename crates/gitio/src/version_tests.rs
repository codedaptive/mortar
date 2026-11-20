use crate::Version;

fn v(major: u32, minor: u32, patch: u32) -> Version {
    Version { major, minor, patch }
}

#[test]
fn orders_by_major_then_minor_then_patch() {
    assert!(v(2, 40, 0) < v(3, 0, 0));
    assert!(v(2, 39, 9) < v(2, 40, 0));
    assert!(v(2, 40, 0) < v(2, 40, 1));
    assert!(v(1, 99, 99) < v(2, 0, 0));
    assert_eq!(v(2, 56, 0), v(2, 56, 0));
}

#[test]
fn is_copy() {
    let a = v(2, 45, 1);
    let b = a;
    assert_eq!(a, b);
}
