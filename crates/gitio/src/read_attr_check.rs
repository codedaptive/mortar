//! read_attr_check: read the answer of `check-attr -z merge -- <paths>`.

use crate::GitError;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

/// NUL-ended fields (the final NUL required) in triples `<path> merge <value>`, exactly one per path in `paths`, in
/// the same order, each path equal to the one sent byte for byte (git echoes it as given: "./a.txt" stays so). The
/// value is non-empty UTF-8: "unspecified", "set", "unset" or the attribute's value. Each path is paired with its
/// value. A count other than one triple per path, another path, another attribute, an empty or non-UTF-8 value, or
/// a missing final NUL is refused naming the output.
pub(crate) fn read_attr_check(stdout: &[u8], paths: &[PathBuf]) -> Result<Vec<(PathBuf, String)>, GitError> {
    let refuse = |why: &str| {
        GitError::refusal(
            Vec::new(),
            format!("not a check-attr answer ({why}): {:?}", String::from_utf8_lossy(stdout)),
        )
    };
    let body = stdout.strip_suffix(b"\0").ok_or_else(|| refuse("no final NUL"))?;
    let fields: Vec<&[u8]> = body.split(|b| *b == 0).collect();
    if fields.len() != 3 * paths.len() {
        return Err(refuse("not one path, merge, value triple per path sent"));
    }
    fields
        .chunks(3)
        .zip(paths)
        .map(|(triple, path)| {
            if triple[0] != path.as_os_str().as_bytes() {
                return Err(refuse("another path"));
            }
            if triple[1] != b"merge" {
                return Err(refuse("another attribute"));
            }
            match std::str::from_utf8(triple[2]) {
                Ok(value) if !value.is_empty() => Ok((path.clone(), value.to_string())),
                _ => Err(refuse("an empty or non-UTF-8 value")),
            }
        })
        .collect()
}
