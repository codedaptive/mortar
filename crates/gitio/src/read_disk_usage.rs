//! read_disk_usage: read the answer of `rev-list --disk-usage`.

use crate::text_refusal::text_refusal;
use crate::{GitError, Output};

/// Exit 0 is one line of ASCII digits, `<bytes>\n`: that number (stderr is not read). Exit 128 with nothing on stdout
/// and stderr exactly `fatal: bad revision '<ref>'\n` for one of the `refs` sent is refused naming that ref (unknown).
/// Any other exit 128 (not a repository, a corrupt ref) is git's own error, returned with its stderr. Anything else,
/// and an exit-0 stdout that is not that one line or does not fit a u64, is refused naming what was found.
pub(crate) fn read_disk_usage<S: AsRef<str>>(output: &Output, refs: &[S]) -> Result<u64, GitError> {
    match output.status {
        0 => bytes(&output.stdout).ok_or_else(|| odd(output)),
        128 if output.stdout.is_empty() => match unknown(&output.stderr, refs) {
            Some(name) => Err(text_refusal("unknown ref", name)),
            None => Err(gits_error(output)),
        },
        128 => Err(gits_error(output)),
        _ => Err(odd(output)),
    }
}

/// The number in `<digits>\n`; None for any other text or a number past u64::MAX. The digit check comes first
/// because u64's parser also takes a leading `+`.
fn bytes(stdout: &[u8]) -> Option<u64> {
    let digits = stdout.strip_suffix(b"\n")?;
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return None;
    }
    std::str::from_utf8(digits).ok()?.parse().ok()
}

/// The sent ref whose bad-revision line is exactly `stderr`. Each name is compared whole, because a name may hold `'`.
fn unknown<'a, S: AsRef<str>>(stderr: &[u8], refs: &'a [S]) -> Option<&'a str> {
    refs.iter()
        .map(AsRef::as_ref)
        .find(|name| stderr == format!("fatal: bad revision '{name}'\n").as_bytes())
}

fn gits_error(output: &Output) -> GitError {
    GitError {
        args: Vec::new(),
        code: Some(128),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

fn odd(output: &Output) -> GitError {
    GitError::refusal(
        Vec::new(),
        format!(
            "rev-list --disk-usage answered exit {} with stdout {:?} and stderr {:?}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    )
}
