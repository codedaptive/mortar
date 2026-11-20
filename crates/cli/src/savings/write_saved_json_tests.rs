//! write_saved_json: baseline, baselineChars, outputChars, in characters, appended to what is there.

use super::{write_saved_json, Savings};

#[test]
fn appends_the_object_in_key_order() {
    let mut out = String::from("x:");
    let savings = Savings { baseline: "git \"status\"".to_string(), baseline_chars: 2_161_007 };
    write_saved_json(&mut out, &savings, 54_368);
    assert_eq!(out, "x:{\"baseline\":\"git \\\"status\\\"\",\"baselineChars\":2161007,\"outputChars\":54368}");
}

#[test]
fn zero_counts() {
    let mut out = String::new();
    write_saved_json(&mut out, &Savings { baseline: String::new(), baseline_chars: 0 }, 0);
    assert_eq!(out, "{\"baseline\":\"\",\"baselineChars\":0,\"outputChars\":0}");
}
