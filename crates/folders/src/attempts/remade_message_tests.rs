//! remade_message: "<request>: attempt <k> re-made from its snapshot".

use super::remade_message;

#[test]
fn names_the_request_and_the_attempt() {
    assert_eq!(remade_message("req-7", 3), "req-7: attempt 3 re-made from its snapshot");
}

#[test]
fn writes_a_many_digit_k_in_full() {
    assert_eq!(remade_message("r", 12), "r: attempt 12 re-made from its snapshot");
}
