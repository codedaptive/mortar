//! squash_message_requests: the requests a squash message names, read back from the request paragraph that
//! squash_message writes. squash_message_requests_tests covers it.

/// The requests of `message`, in message order. The message is split into paragraphs at blank lines (lines empty
/// after trim_end; a run of them is one break). The last paragraph is the trailer block; the request paragraph is
/// the one just before it, and never the first (the subject's). When every line of that paragraph starts with
/// "- ", each line names one request, the text between "- " and its first `": "`; otherwise there are no requests.
/// A first line of the form `<zone>: <n> requests` (Mortar's subject, n read after the last `": "`) must count the
/// requests exactly; any other first line is a caller's subject and is not counted. None when a request line has
/// no `": "` or Mortar's count does not match.
///
/// Known limit: a caller's message with no requests whose last body paragraph is all `- x: y` lines reads those
/// lines back as requests; the squash message has no other place to mark its request lines. And a caller's
/// subject that itself has the form `<x>: <n> requests` is counted as Mortar's, so it reads back as None unless n
/// matches its request lines.
pub(crate) fn squash_message_requests(message: &str) -> Option<Vec<String>> {
    let lines: Vec<&str> = message.lines().collect();
    let paragraphs: Vec<&[&str]> =
        lines.split(|line| line.trim_end().is_empty()).filter(|paragraph| !paragraph.is_empty()).collect();
    let marked: Vec<&str> = match paragraphs.len() {
        n if n >= 3 => paragraphs[n - 2].iter().map(|line| line.strip_prefix("- ")).collect::<Option<_>>(),
        _ => None,
    }
    .unwrap_or_default();
    let requests = marked
        .iter()
        .map(|rest| rest.split_once(": ").map(|(request, _)| request.to_string()))
        .collect::<Option<Vec<String>>>()?;
    match paragraphs.first().and_then(|subject| subject_count(subject[0])) {
        Some(count) if count != requests.len() => None,
        _ => Some(requests),
    }
}

/// n of a subject `<zone>: <n> requests`, read after its last `": "`; None for any other subject.
fn subject_count(subject: &str) -> Option<usize> {
    subject.rsplit_once(": ")?.1.strip_suffix(" requests")?.parse().ok()
}
