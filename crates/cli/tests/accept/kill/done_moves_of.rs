//! The done records that moved one ref since a given op began.

use super::OpLine;

/// Each "done" record holding a move of `refname`, in log order, from the first record of op `since` on (the
/// killed op's intent), so the killed op's own closing record counts and nothing before the kill does. With no
/// record of `since`, the whole log.
pub fn done_moves_of<'a>(lines: &'a [OpLine], refname: &str, since: &str) -> Vec<&'a OpLine> {
    let from = lines.iter().position(|line| line.op_id == since).unwrap_or(0);
    lines[from..]
        .iter()
        .filter(|line| line.phase == "done" && line.moves.iter().any(|(name, _)| name == refname))
        .collect()
}
