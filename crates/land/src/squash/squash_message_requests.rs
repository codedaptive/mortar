//! squash_message_requests: the requests a squash message names, read back from the lines squash_message writes.
//! squash_message_requests_tests covers it.

/// The requests of `message`, in message order. Its first line must be `<zone>: <n> requests`, n read after the
/// last `": "`. Each later line starting with "- " names one request, the text before its first `": "`; the
/// trailer paragraph and blank lines never start with "- ". None when the subject is not of that form, a "- " line
/// has no `": "`, or the number of request lines is not n.
pub(crate) fn squash_message_requests(message: &str) -> Option<Vec<String>> {
    let mut lines = message.lines();
    let count: usize = lines.next()?.rsplit_once(": ")?.1.strip_suffix(" requests")?.parse().ok()?;
    let requests = lines
        .filter_map(|line| line.strip_prefix("- "))
        .map(|rest| rest.split_once(": ").map(|(request, _)| request.to_string()))
        .collect::<Option<Vec<String>>>()?;
    (requests.len() == count).then_some(requests)
}
