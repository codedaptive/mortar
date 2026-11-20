//! The environment that turns off git's index refresh for one run.

use std::ffi::{OsStr, OsString};

/// GIT_CONFIG_COUNT n+1 with diff.autoRefreshIndex=false as config pair n, where n is `config_count` (the caller's
/// GIT_CONFIG_COUNT) when it is a non-negative integer, else 0. The pair goes at index n so the caller's own pairs
/// 0..n-1 stay in force. n is read as a u32, so n+1 cannot overflow; a count past u32 is no count git accepts.
pub fn no_index_refresh_env(config_count: Option<&OsStr>) -> Vec<(OsString, OsString)> {
    let n = config_count.and_then(OsStr::to_str).and_then(|count| count.parse::<u32>().ok()).unwrap_or(0);
    vec![
        ("GIT_CONFIG_COUNT".into(), (u64::from(n) + 1).to_string().into()),
        (format!("GIT_CONFIG_KEY_{n}").into(), "diff.autoRefreshIndex".into()),
        (format!("GIT_CONFIG_VALUE_{n}").into(), "false".into()),
    ]
}
