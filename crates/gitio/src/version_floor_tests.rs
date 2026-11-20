use crate::Version;

#[test]
fn floor_is_2_46_0() {
    assert_eq!(
        Version::FLOOR,
        Version {
            major: 2,
            minor: 46,
            patch: 0
        }
    );
}
