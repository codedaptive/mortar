//! hook_refusal_text: the sentence naming a commit hook that refused, shared by SaveHooksError's and
//! CommitPathsHooksError's Display. hook_refusal_text_tests.rs checks its text.

use gitio::HookRefusal;

/// Names the hook and its exit code, or says a signal killed it. The hook's stdout and stderr are left out: the
/// caller shows them as the hook wrote them.
pub(crate) fn hook_refusal_text(refusal: &HookRefusal) -> String {
    match refusal.code {
        Some(code) => format!("the {} hook refused the commit with exit code {code}", refusal.hook),
        None => format!("the {} hook was killed by a signal", refusal.hook),
    }
}
