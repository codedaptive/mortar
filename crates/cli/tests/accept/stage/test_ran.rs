//! The child the stage tests read replies from. Test-only: read_reply_tests.rs, reply_is_tests.rs,
//! reply_text_tests.rs, reply_texts_tests.rs and setup_step_tests.rs build their input with it.

use crate::world::Ran;

/// A child that ended with `code` and printed `stdout`, with no signal and no stderr.
pub fn test_ran(code: Option<i32>, stdout: &str) -> Ran {
    Ran { code, signal: None, stdout: stdout.to_string(), stderr: String::new() }
}
