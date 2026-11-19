//! Config setup through Cargo's actual mortar binary, shared by the test fixtures. No private cli API or codec.
use std::{path::Path, process::{Command, Stdio}};
use super::Config;
use crate::json::check;

/// Supplies every fixture option through `init --replace`; accepts its three successful write/repeat outcomes.
/// An empty build-dir list uses init's documented default: these fixtures create no build output to exclude.
pub fn write_fixture_config(root: &Path, config: &Config) -> Result<(), String> {
    // End global parsing first: a formatter argument such as "--" or "--json" belongs to init's value words.
    let mut words = vec!["--json".to_string(), "--actor".into(), "rit-config".into(), "init".into(), "--".into(), "--replace".into()];
    for (flag, value) in [
        ("--origin", config.origin_branch.clone()), ("--integration", config.integration_branch.clone()),
        ("--remote", config.remote.clone()), ("--cap-bytes", config.cap_bytes.to_string()),
        ("--floor-bytes", config.floor_bytes.to_string()),
    ] { words.extend([flag.to_string(), value]); }
    for (zone, paths) in &config.zones {
        words.extend(["--zone".into(), format!("{zone}={}", paths.paths.join(","))]);
    }
    for (flag, values) in [("--root-file", &config.root_files), ("--build-dir", &config.build_dirs)] {
        for value in values { words.extend([flag.to_string(), value.clone()]); }
    }
    if let Some(formatter) = &config.formatter {
        for word in &formatter.command { words.extend(["--formatter".into(), word.clone()]); }
        words.extend(["--formatter-timeout".into(), formatter.timeout_seconds.to_string()]);
    }
    let output = Command::new(env!("CARGO_BIN_EXE_mortar")).current_dir(root).env_clear()
        .env("PATH", std::env::var_os("PATH").unwrap()).env("HOME", root).env("TMPDIR", root)
        .env("GIT_CONFIG_NOSYSTEM", "1").env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("CODETEAM_SESSION", "harness").stdin(Stdio::null()).args(&words).output()
        .map_err(|error| format!("init fixture: {error}"))?;
    if !output.status.success() { return Err(format!("init fixture failed: {output:?}")); }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let outcome = ["written", "already-done", "replaced"].into_iter()
        .find(|name| stdout.contains(&format!("\"outcome\":\"{name}\"")))
        .ok_or_else(|| format!("unexpected init fixture reply: {output:?}"))?;
    check(output, "init", 0, outcome);
    Ok(())
}
