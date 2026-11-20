//! Whether what a refused git command printed gives the guard's refusal as step 2 asks: why, owner and next.

use super::refusal_parts;

/// Who may move a guarded ref, as the guard names it (land/src/guard/guard_decide.rs).
const OWNER: &str = "the landing surface: mr land and mr publish";

/// What to do instead, as the guard says it (land/src/guard/guard_decide.rs).
const NEXT: &str = "bring your work in with `mr join`, or ask for a landing";

/// True when refusal_parts reads `why` exactly, the guard's owner and its next from `output`.
pub fn refusal_ok(output: &str, why: &str) -> bool {
    let (seen, owner, next) = refusal_parts(output);
    seen == why && owner == OWNER && next == NEXT
}
