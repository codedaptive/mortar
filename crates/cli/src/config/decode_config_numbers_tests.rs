//! decode_config on numbers outside the config's integers, on booleans in any value position and on torn
//! literals. In the text `with` builds, the cap_bytes value starts at byte 159 and the root_files value at 120.

use super::decode_config;
use super::decode_config_tests::{reason, with};

#[test]
fn numbers_out_of_the_subset_are_refused_at_their_byte() {
    assert_eq!(reason(&with("cap_bytes", "012")), "number 012 at byte 159 has a leading zero");
    assert_eq!(
        reason(&with("root_files", "[1.5]")),
        "number at byte 121 has a fraction or exponent; only integers are allowed"
    );
    assert_eq!(
        reason(&with("cap_bytes", "2e3")),
        "number at byte 159 has a fraction or exponent; only integers are allowed"
    );
    assert_eq!(
        reason(&with("cap_bytes", "18446744073709551616")),
        "number 18446744073709551616 at byte 159 does not fit 64 bits"
    );
    assert_eq!(
        reason(&with("cap_bytes", "-1")),
        "unexpected '-' at byte 159 where a value (object, array, string, null, boolean or non-negative integer) belongs"
    );
}

#[test]
fn the_largest_u64_is_a_number() {
    assert!(decode_config(&with("cap_bytes", "18446744073709551615")).is_ok_and(|c| c.cap_bytes == u64::MAX));
}

#[test]
fn a_boolean_is_refused_by_the_key_that_holds_it() {
    assert_eq!(reason(&with("cap_bytes", "true")), "cap_bytes is not a number");
    assert_eq!(reason(&with("remote", "false")), "remote is not a string");
    assert_eq!(reason(&with("build_dirs", "[true]")), "build_dirs item 0 is not a string");
    assert_eq!(reason(&with("zones", "false")), "zones is not an object");
    assert_eq!(reason(&with("zones", r#"{"a":true}"#)), "zones.a is not an object");
    assert_eq!(reason(&with("zones", r#"{"a":{"paths":[false]}}"#)), "zones.a.paths item 0 is not a string");
    assert_eq!(reason(&with("formatter", "true")), "formatter is neither null nor an object");
    assert_eq!(
        reason(&with("formatter", r#"{"command":["f"],"timeout_seconds":false}"#)),
        "formatter.timeout_seconds is not a number"
    );
    assert_eq!(reason(&with("formatter", r#"{"command":[true],"timeout_seconds":1}"#)), "formatter.command item 0 is not a string");
}

#[test]
fn a_top_level_boolean_is_not_an_object() {
    assert_eq!(reason("true"), "the config is not an object");
    assert_eq!(reason(" false "), "the config is not an object");
}

#[test]
fn a_torn_literal_is_text_that_ends_early() {
    for text in ["tr", "fals", r#"{"origin_branch":"main","cap_bytes":tr"#, r#"{"remote":fals"#] {
        assert_eq!(reason(text), "the text ends before the config is complete", "{text:?}");
    }
}
