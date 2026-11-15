//! decide_with_intent: land::guard_decide's verdict on a hook run, reading the open intent's moves only when the
//! verdict can depend on them. decide_with_intent_tests covers it with land::guard_decide itself.

use land::{guard_decide, GuardHook, GuardVerdict, LandRefs};
use oplog::RefMove;

/// A line no hook's input parser can read: non-empty, so guard_decide does not drop it as it drops empty lines, and
/// with no space, so it has neither the 3 fields of a transaction line nor the 4 of a pre-push line.
const UNREADABLE: &str = "unreadable";

/// guard_decide's verdict on `hook` with `input`, given the moves `read_moves` reads of the open intent.
/// guard_decide refuses a line it cannot read whenever it reads a hook's input, so a hook that lets UNREADABLE
/// through reads none of its input: no intent can change its verdict, and `read_moves` is not called. Otherwise
/// `read_moves` is called once and its error comes back as is.
pub fn decide_with_intent<E>(
    hook: &GuardHook,
    input: &str,
    refs: &LandRefs,
    read_moves: impl FnOnce() -> Result<Option<Vec<RefMove>>, E>,
) -> Result<GuardVerdict, E> {
    if guard_decide(hook, UNREADABLE, refs, None) == GuardVerdict::Allow {
        return Ok(guard_decide(hook, input, refs, None));
    }
    Ok(guard_decide(hook, input, refs, read_moves()?.as_deref()))
}
