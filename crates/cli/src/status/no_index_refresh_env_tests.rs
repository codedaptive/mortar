//! no_index_refresh_env: diff.autoRefreshIndex=false added as config pair n after the caller's n pairs.

use std::ffi::{OsStr, OsString};

use super::no_index_refresh_env;

fn env(pairs: [(&str, &str); 3]) -> Vec<(OsString, OsString)> {
    pairs.iter().map(|(key, value)| (OsString::from(key), OsString::from(value))).collect()
}

#[test]
fn no_count_is_pair_zero() {
    assert_eq!(
        no_index_refresh_env(None),
        env([
            ("GIT_CONFIG_COUNT", "1"),
            ("GIT_CONFIG_KEY_0", "diff.autoRefreshIndex"),
            ("GIT_CONFIG_VALUE_0", "false"),
        ])
    );
}

#[test]
fn a_count_of_zero_is_pair_zero() {
    assert_eq!(
        no_index_refresh_env(Some(OsStr::new("0"))),
        env([
            ("GIT_CONFIG_COUNT", "1"),
            ("GIT_CONFIG_KEY_0", "diff.autoRefreshIndex"),
            ("GIT_CONFIG_VALUE_0", "false"),
        ])
    );
}

#[test]
fn a_count_of_two_adds_only_pair_two_and_raises_the_count_to_three() {
    assert_eq!(
        no_index_refresh_env(Some(OsStr::new("2"))),
        env([
            ("GIT_CONFIG_COUNT", "3"),
            ("GIT_CONFIG_KEY_2", "diff.autoRefreshIndex"),
            ("GIT_CONFIG_VALUE_2", "false"),
        ])
    );
}

#[test]
fn an_unparsable_count_is_pair_zero() {
    for count in ["x", "-1", ""] {
        assert_eq!(
            no_index_refresh_env(Some(OsStr::new(count))),
            env([
                ("GIT_CONFIG_COUNT", "1"),
                ("GIT_CONFIG_KEY_0", "diff.autoRefreshIndex"),
                ("GIT_CONFIG_VALUE_0", "false"),
            ]),
            "{count}"
        );
    }
}
