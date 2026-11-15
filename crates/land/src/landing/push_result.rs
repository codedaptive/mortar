//! PushResult: what push_landing's push came to. A bare type (S-7); the publish_zit test
//! publish_moves_the_remote_then_the_local_origin_and_cleans_up exercises Pushed and
//! a_remote_moved_while_the_push_was_refused_answers_origin_moved Moved.

/// Pushed: the remote's origin branch is at or past the landed commit (pushed now, or found there after a refused
/// push). Moved: the push did not go through and the remote's origin branch is neither at the landing's starting
/// origin nor at or past the landed commit (whether git's lease or the guard refused it).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PushResult {
    Pushed,
    Moved,
}
