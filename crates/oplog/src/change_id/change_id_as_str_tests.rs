//! as_str: the ID's 32 chars as written.

use super::ChangeId;

#[test]
fn gives_the_text() {
    let mut bytes = [0u8; 16];
    bytes[0] = 0x9c;
    assert_eq!(ChangeId::from_bytes(bytes).as_str(), "qnzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz");
}
