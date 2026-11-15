//! push_landed: whether the push of the landing reached the remote, from what the remote's origin branch holds (S-103,
//! S-29).
//! No unit test (S-5): past the cheap cases it fetches from the remote and asks git about ancestry. Covered by the
//! publish_zit tests publishing::an_open_push_intent_with_the_remote_past_new_is_closed_done (case 9: the remote
//! past new), publishing::an_open_push_intent_after_the_push_is_closed_done_on_the_repeat (case 7: at new) and
//! publishing::an_open_push_intent_before_the_push_is_closed_failed_and_pushed_again (case 8: at expected).

use crate::landing::PublishError;
use crate::squash::{read_tip, LandRefs};
use gitio::{FetchRefOutcome, Git, Oid};

/// `remote` at `new`: true. Absent or at `expected`: false. Anywhere else, the remote's `refs/heads/<origin_branch>`
/// is fetched onto `refs/mortar/fetched/<origin_branch>` (gone from the remote by then: false) and the answer is
/// whether `new` is its ancestor: the origin only moves forward (S-29), so a remote past new holds our push with
/// someone else's work on top. Only this case reaches the remote beyond the ls-remote publish already made.
pub(crate) fn push_landed(
    git: &Git,
    refs: &LandRefs,
    remote: Option<&Oid>,
    expected: &Oid,
    new: &Oid,
) -> Result<bool, PublishError> {
    match remote {
        Some(remote) if remote == new => return Ok(true),
        None => return Ok(false),
        Some(remote) if remote == expected => return Ok(false),
        Some(_) => {}
    }
    let origin_ref = format!("refs/heads/{}", refs.origin_branch);
    let fetched_ref = format!("refs/mortar/fetched/{}", refs.origin_branch);
    let outcome = git.fetch_ref(&refs.remote, &origin_ref, &fetched_ref).map_err(PublishError::Git)?;
    if outcome == FetchRefOutcome::MissingSource {
        return Ok(false);
    }
    let fetched = read_tip(git, &fetched_ref).map_err(PublishError::Squash)?;
    git.is_ancestor(new, &fetched).map_err(PublishError::Git)
}
