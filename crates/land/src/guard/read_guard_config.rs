//! read_guard_config: the branches the guard protects, as guard.json names them. It runs git and reads a file, so
//! it has no unit test (S-5). tests/land_rit/guard_codec.rs reads present files and reaches these BadConfig refusals,
//! each naming the path, and no others: a missing, repeated or unknown key; a key that is not a string; no ':' after
//! a key; a value that is not a string, has no closing quote or holds a raw control char; no ',' or '}' after a
//! member; text after the object. Owed to Nagatha as a RIT: a missing guard.json giving Ok(None) (case 8 in
//! ledger/reports/WO-land-guard-symref-for-nagatha.md).

use gitio::Git;

use super::guard_config_path::guard_config_path;
use super::parse_guard_config::parse_guard_config;
use super::read_optional::read_optional;
use super::GuardError;
use crate::LandRefs;

/// guard.json under `git`'s common dir: None when the guard is not installed; BadConfig naming the file when its
/// text is not UTF-8 or not a guard config. `mortar guard-hook` reads this before guard_decide.
pub fn read_guard_config(git: &Git) -> Result<Option<LandRefs>, GuardError> {
    let path = guard_config_path(&git.common_dir().map_err(GuardError::Git)?);
    let Some(bytes) = read_optional(&path)? else {
        return Ok(None);
    };
    let parsed = String::from_utf8(bytes)
        .map_err(|e| format!("it is not UTF-8: {e}"))
        .and_then(|text| parse_guard_config(&text));
    parsed.map(Some).map_err(|why| GuardError::BadConfig { path, why })
}
