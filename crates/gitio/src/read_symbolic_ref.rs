//! read_symbolic_ref: read the answer of `symbolic-ref -q HEAD`.

use crate::text_refusal::text_refusal;
use crate::{GitError, Output};

/// Exit 0 is exactly one non-empty UTF-8 line ending in `\n` with no other line break: the full ref name HEAD points
/// at (Some). Exit 1 with nothing on stdout or stderr is a detached HEAD (None). Any other text is refused naming
/// it, and any other exit is refused naming the exit, stdout and stderr.
pub(crate) fn read_symbolic_ref(output: &Output) -> Result<Option<String>, GitError> {
    match (output.status, output.stdout.is_empty(), output.stderr.is_empty()) {
        (0, _, _) => branch_line(&output.stdout).map(Some),
        (1, true, true) => Ok(None),
        (status, _, _) => Err(GitError::refusal(
            Vec::new(),
            format!(
                "symbolic-ref answered exit {status} with stdout {:?} and stderr {:?}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
        )),
    }
}

/// The name from the one line `<name>\n`; anything else is refused naming the text (lossy when not UTF-8).
fn branch_line(stdout: &[u8]) -> Result<String, GitError> {
    let Ok(text) = std::str::from_utf8(stdout) else {
        return Err(text_refusal("not a UTF-8 symbolic-ref line", &String::from_utf8_lossy(stdout)));
    };
    match text.strip_suffix('\n') {
        Some(name) if !name.is_empty() && !name.contains('\n') => Ok(name.to_string()),
        _ => Err(text_refusal("not one symbolic-ref line", text)),
    }
}
