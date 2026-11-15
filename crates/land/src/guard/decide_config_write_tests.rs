//! decide_config_write: absent, same bytes and different bytes, with literal bytes.

use super::decide_config_write::decide_config_write;

#[test]
fn absent_config_is_written() {
    assert!(decide_config_write(None, "{\"zone\":\"land\"}\n"));
}

#[test]
fn same_bytes_are_left_alone() {
    assert!(!decide_config_write(Some(b"{\"zone\":\"land\"}\n"), "{\"zone\":\"land\"}\n"));
}

#[test]
fn different_bytes_are_rewritten() {
    assert!(decide_config_write(Some(b"{\"zone\":\"join\"}\n"), "{\"zone\":\"land\"}\n"));
}

#[test]
fn a_trailing_byte_more_is_rewritten() {
    assert!(decide_config_write(Some(b"{\"zone\":\"land\"}\n\n"), "{\"zone\":\"land\"}\n"));
}
