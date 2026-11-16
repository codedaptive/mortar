//! A step's verdict from what it recorded.

use super::{Line, Verdict};

/// Fail when the body panicked, when any check did not hold, or when there is no check at all (a step that
/// checked nothing has not passed); else Pass. Log lines never decide.
pub fn block_verdict(lines: &[Line], panic: Option<&str>) -> Verdict {
    let mut checked = false;
    for line in lines {
        match line {
            Line::Check { ok: false, .. } => return Verdict::Fail,
            Line::Check { ok: true, .. } => checked = true,
            Line::Log(_) => {}
        }
    }
    if panic.is_some() || !checked {
        return Verdict::Fail;
    }
    Verdict::Pass
}
