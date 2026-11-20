use crate::PatchId;

#[test]
fn as_str_is_the_parsed_hex() {
    let hex = "fedcba9876543210fedcba9876543210fedcba98";
    assert_eq!(PatchId::parse(hex).unwrap().as_str(), "fedcba9876543210fedcba9876543210fedcba98");
}
