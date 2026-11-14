//! guard_hook_from_args: the words after `mortar guard-hook`, read as the hook git ran.

use land::GuardHook;

/// `["reference-transaction", <state>]` or `["pre-push", <remote>, <url>]`, the words git passes each hook. For
/// pre-push, `push_urls` gives the guarded remote's push URLs (None when they cannot be read); it is called only for
/// pre-push. No hook name, an unknown one, or the wrong number of words for the hook is Err with the reason.
pub fn guard_hook_from_args(
    rest: &[String],
    push_urls: impl FnOnce() -> Option<Vec<String>>,
) -> Result<GuardHook, String> {
    let Some((name, words)) = rest.split_first() else {
        return Err("guard-hook takes the hook's name: reference-transaction or pre-push".to_string());
    };
    match (name.as_str(), words) {
        ("reference-transaction", [state]) => Ok(GuardHook::ReferenceTransaction { state: state.clone() }),
        ("pre-push", [remote, url]) => {
            Ok(GuardHook::PrePush { remote: remote.clone(), url: url.clone(), push_urls: push_urls() })
        }
        ("reference-transaction", _) => {
            Err(format!("reference-transaction takes one word, the state, and was given {}", words.len()))
        }
        ("pre-push", _) => {
            Err(format!("pre-push takes two words, the remote and the url, and was given {}", words.len()))
        }
        (other, _) => Err(format!("unknown hook {other:?}: the hooks are reference-transaction and pre-push")),
    }
}
