use std::ffi::{OsStr, OsString};
use std::path::Path;

use super::world_env;

fn value(env: &[(OsString, OsString)], key: &str) -> Option<String> {
    env.iter().find(|(k, _)| k == key).map(|(_, v)| v.to_string_lossy().into_owned())
}

#[test]
#[ignore = "acceptance: make accept"]
fn git_config_is_cut_off_and_home_is_the_root() {
    let env = world_env(Path::new("/scratch/root"), OsStr::new("/usr/bin:/bin"));
    assert_eq!(value(&env, "GIT_CONFIG_GLOBAL").as_deref(), Some("/dev/null"));
    assert_eq!(value(&env, "GIT_CONFIG_NOSYSTEM").as_deref(), Some("1"));
    assert_eq!(value(&env, "HOME").as_deref(), Some("/scratch/root"));
    assert_eq!(value(&env, "TMPDIR").as_deref(), Some("/scratch/root"));
    assert_eq!(value(&env, "PATH").as_deref(), Some("/usr/bin:/bin"));
    assert_eq!(value(&env, "GIT_AUTHOR_EMAIL").as_deref(), Some("accept@example.invalid"));
}

#[test]
#[ignore = "acceptance: make accept"]
fn only_the_listed_variables_are_set() {
    let env = world_env(Path::new("/r"), OsStr::new("/bin"));
    let mut keys: Vec<String> = env.iter().map(|(k, _)| k.to_string_lossy().into_owned()).collect();
    keys.sort();
    assert_eq!(
        keys,
        [
            "GIT_AUTHOR_EMAIL", "GIT_AUTHOR_NAME", "GIT_COMMITTER_EMAIL", "GIT_COMMITTER_NAME", "GIT_CONFIG_GLOBAL",
            "GIT_CONFIG_NOSYSTEM", "GIT_TERMINAL_PROMPT", "HOME", "LC_ALL", "PATH", "TMPDIR",
        ]
    );
}
