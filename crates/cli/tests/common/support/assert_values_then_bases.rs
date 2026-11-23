//! The pin on an op record's values that also accepts join's trailing base values, shared by the test crates.

/// `values` starts with exactly `expected`, then holds nothing more, or non-empty base_before then base_after.
pub fn assert_values_then_bases(values: &[(String, String)], expected: &[(String, String)]) {
    assert_eq!(values.get(..expected.len()), Some(expected), "{values:?}");
    let tail: Vec<_> = values[expected.len()..].iter().map(|(name, value)| { assert!(!value.is_empty(), "{values:?}"); name.as_str() }).collect();
    assert!(tail.is_empty() || tail == ["base_before", "base_after"], "{values:?}");
}
