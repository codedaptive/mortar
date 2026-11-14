//! squash_message: the text of a squash commit's message, before its trailers.

use crate::squash::SquashRequest;

/// Subject `<zone>: <n> requests` (the spec's text for every n), then a blank line and one line per request,
/// `- <request>: <subject>; <subject>`, each ended by a newline. No requests: the subject line alone.
/// Why the "- ": this text goes to git's with_trailers, and a last paragraph of `<request>: <subject>` lines has
/// git's trailer form, so git would take it as the message's trailer block and add the Mortar trailers into it.
/// "- " makes the token before the colon hold a space, which git never reads as a trailer key, so the Mortar
/// trailers get a paragraph of their own.
pub(crate) fn squash_message(zone: &str, requests: &[SquashRequest]) -> String {
    let subject = format!("{zone}: {} requests\n", requests.len());
    if requests.is_empty() {
        return subject;
    }
    let lines: String =
        requests.iter().map(|group| format!("- {}: {}\n", group.request, group.subjects.join("; "))).collect();
    format!("{subject}\n{lines}")
}
