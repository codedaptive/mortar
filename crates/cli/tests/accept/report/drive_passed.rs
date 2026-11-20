//! Whether the drive passed, from its blocks.

use super::{Block, Verdict};

/// True when no block failed. A NOT RUN block does not fail the drive: its reason says what to set.
pub fn drive_passed(blocks: &[Block]) -> bool {
    !blocks.iter().any(|block| block.verdict == Verdict::Fail)
}
