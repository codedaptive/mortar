//! parse_install_args: `mr install`'s arguments.

use super::{find_harness, HarnessSpec, InstallOptions, SkillLocation, HARNESSES};

/// The options `rest` asks for, or the usage reason (exit 2 by the caller). Switches: `--no-place`,
/// `--short-name`, `-y`/`--yes`. Valued flags, as `--flag <v>` or `--flag=<v>`: `--target` (a comma list of
/// harnesses), `--mode` (skill only), `--location` (global or local). A flag given twice, a missing or empty value,
/// or any other word is a usage error.
pub fn parse_install_args(rest: &[String]) -> Result<InstallOptions, String> {
    let mut options = InstallOptions {
        no_place: false,
        short_name: false,
        targets: None,
        yes: false,
        location: SkillLocation::Global,
    };
    let mut seen: Vec<&str> = Vec::new();
    let mut words = rest.iter();
    while let Some(word) = words.next() {
        let (flag, inline) = match word.split_once('=') {
            Some((flag, value)) if flag.starts_with("--") => (flag, Some(value)),
            _ => (word.as_str(), None),
        };
        let unexpected = || format!("unexpected word {word:?} in install");
        // -y and --yes are one flag, so either after the other is the flag given twice.
        let key = if flag == "-y" { "--yes" } else { flag };
        if !["--no-place", "--short-name", "--yes", "--target", "--mode", "--location"].contains(&key) {
            return Err(unexpected());
        }
        if seen.contains(&key) {
            return Err(format!("{flag} is given twice"));
        }
        seen.push(key);
        match (key, inline) {
            ("--no-place", None) => options.no_place = true,
            ("--short-name", None) => options.short_name = true,
            ("--yes", None) => options.yes = true,
            ("--no-place" | "--short-name" | "--yes", Some(_)) => return Err(unexpected()),
            _ => {
                let value = match inline.or_else(|| words.next().map(String::as_str)) {
                    None | Some("") => return Err(format!("{flag} needs a value")),
                    Some(value) => value,
                };
                apply_value(&mut options, key, value)?;
            }
        }
    }
    Ok(options)
}

/// Sets the option a valued flag names from its value, or the usage reason the value is not one of its words.
fn apply_value(options: &mut InstallOptions, flag: &str, value: &str) -> Result<(), String> {
    match (flag, value) {
        ("--target", list) => options.targets = Some(parse_targets(list)?),
        ("--mode", "skill") => {}
        ("--mode", "mcp" | "both") => {
            return Err("Mortar offers --mode skill only; mcp and both are Tessera's".to_string())
        }
        ("--mode", other) => return Err(format!("unknown mode {other:?}: Mortar offers skill")),
        ("--location", "global") => options.location = SkillLocation::Global,
        ("--location", "local") => options.location = SkillLocation::Local,
        (_, other) => return Err(format!("unknown location {other:?}: global or local")),
    }
    Ok(())
}

/// --target's harnesses, each once, in HARNESSES order.
fn parse_targets(list: &str) -> Result<Vec<&'static HarnessSpec>, String> {
    let mut named = Vec::new();
    for name in list.split(',') {
        if name.is_empty() {
            return Err("--target has an empty name".to_string());
        }
        named.push(find_harness(name).ok_or_else(|| format!("unknown harness {name:?}: claude, codex or cursor"))?);
    }
    Ok(HARNESSES.iter().filter(|harness| named.contains(harness)).collect())
}
