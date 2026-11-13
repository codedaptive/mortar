//! parse: accepts exactly 32 chars of k..z, keeping the text; refuses any other length or char with a
//! Malformed naming the text and saying a change ID is 32 letters k..z.

use super::{ChangeId, ChangeIdError};

const VALID: &str = "kzlymxnwovpuqtrssrtqupvowxnymlzk";

fn assert_refused(text: &str, what: &str) {
    match ChangeId::parse(text) {
        Err(err @ ChangeIdError::Malformed { .. }) => {
            let message = err.to_string();
            assert!(message.contains(&format!("{text:?}")), "{message} lacks the text");
            assert!(message.contains("32 letters k..z"), "{message} lacks the form");
            assert!(message.contains(what), "{message} lacks {what:?}");
        }
        other => panic!("{text:?} gave {other:?}, not Malformed"),
    }
}

#[test]
fn accepts_32_letters_k_to_z_keeping_text() {
    assert_eq!(ChangeId::parse(VALID).unwrap().as_str(), VALID);
}

#[test]
fn refuses_empty() {
    assert_refused("", "0 chars");
}

#[test]
fn refuses_31_chars() {
    assert_refused("kzlymxnwovpuqtrssrtqupvowxnymlz", "31 chars");
}

#[test]
fn refuses_33_chars() {
    assert_refused("kzlymxnwovpuqtrssrtqupvowxnymlzkk", "33 chars");
}

#[test]
fn refuses_hex_digit() {
    assert_refused("kzlymxnwovpuqtrssrtqupvowxnyml0k", "'0'");
}

#[test]
fn refuses_uppercase_k() {
    assert_refused("Kzlymxnwovpuqtrssrtqupvowxnymlzk", "'K'");
}

#[test]
fn refuses_j() {
    assert_refused("jzlymxnwovpuqtrssrtqupvowxnymlzk", "'j'");
}

#[test]
fn refuses_brace_after_z() {
    assert_refused("kzlymxnwovpuqtrssrtqupvowxnymlz{", "'{'");
}

#[test]
fn refuses_multi_byte_char() {
    assert_refused("kzlymxnwovpuqtrssrtqupvowxnymlzé", "'é'");
}
