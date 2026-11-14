//! squash_requests: the commits of a zone grouped by the request each belongs to.

use crate::squash::SquashRequest;

/// `commits` is (subject, Mortar-Request value) for each commit, oldest first. A commit with no request is
/// left out. Requests come in the order of each one's first commit; a request's subjects in commit order.
pub(crate) fn squash_requests(commits: &[(String, Option<String>)]) -> Vec<SquashRequest> {
    let mut groups: Vec<SquashRequest> = Vec::new();
    for (subject, request) in commits {
        let Some(request) = request else { continue };
        match groups.iter_mut().find(|group| &group.request == request) {
            Some(group) => group.subjects.push(subject.clone()),
            None => groups.push(SquashRequest { request: request.clone(), subjects: vec![subject.clone()] }),
        }
    }
    groups
}
