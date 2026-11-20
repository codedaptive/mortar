use crate::Oid;

#[test]
fn display_prints_the_hex() {
    let oid = Oid::parse("89abcdef0123456789abcdef0123456789abcdef").unwrap();
    assert_eq!(oid.to_string(), "89abcdef0123456789abcdef0123456789abcdef");
    assert_eq!(format!("<{oid}>"), "<89abcdef0123456789abcdef0123456789abcdef>");
}
