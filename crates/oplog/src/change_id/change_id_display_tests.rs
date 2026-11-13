//! Display: the ID prints as its 32 chars and nothing else.

use super::ChangeId;

#[test]
fn prints_the_32_chars() {
    let id = ChangeId::parse("kzlymxnwovpuqtrssrtqupvowxnymlzk").unwrap();
    assert_eq!(format!("<{id}>"), "<kzlymxnwovpuqtrssrtqupvowxnymlzk>");
}
