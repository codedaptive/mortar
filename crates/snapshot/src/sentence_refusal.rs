//! sentence_refusal: S-41's save-sentence check, exported so the cli can refuse a sentence before a save (S-90).

/// Why `sentence` cannot be a save commit's subject, or None when it can. An empty or whitespace-only sentence
/// is refused first; otherwise one holding a line break ('\n' or '\r') is refused, since a subject is one line.
pub fn sentence_refusal(sentence: &str) -> Option<&'static str> {
    if sentence.trim().is_empty() {
        Some("the sentence is empty or only whitespace")
    } else if sentence.contains(['\n', '\r']) {
        Some("the sentence holds a line break; a commit subject is one line")
    } else {
        None
    }
}
