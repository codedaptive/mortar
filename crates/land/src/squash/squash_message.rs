//! squash_message: the text of a squash commit's message, before its trailers.
//! squash_message_requests reads its request lines back.

use crate::squash::SquashRequest;

/// The message's first part is the caller's text, trimmed, ended by a newline; with no caller text, or text that
/// is only whitespace, it is Mortar's subject `<zone>: <n> requests` ("requests" for every n, 1 included). Then a
/// blank line and one line per request, `- <request>: <subject>; <subject>`, each ended by a newline. No requests:
/// the first part alone.
/// Why the "- ": this text goes to git's with_trailers, and a last paragraph of `<request>: <subject>` lines has
/// git's trailer form, so git would take it as the message's trailer block and add the Mortar trailers into it.
/// "- " makes the token before the colon hold a space, which git never reads as a trailer key, so the Mortar
/// trailers get a paragraph of their own.
pub(crate) fn squash_message(zone: &str, requests: &[SquashRequest], caller: Option<&str>) -> String {
    let head = match caller.map(str::trim).filter(|text| !text.is_empty()) {
        Some(text) => format!("{text}\n"),
        None => format!("{zone}: {} requests\n", requests.len()),
    };
    if requests.is_empty() {
        return head;
    }
    let lines: String =
        requests.iter().map(|group| format!("- {}: {}\n", group.request, group.subjects.join("; "))).collect();
    format!("{head}\n{lines}")
}
