//! publish: puts a landing on the remote and then on the local origin branch, push on; publish_with_push has the
//! steps. It takes no lock: the caller holds `land` and has run recover.
//! No unit test: it answers through publish_with_push, which runs git; decide_publish_tests covers its decisions.
//! Covered by the publish_zit tests publish_moves_the_remote_then_the_local_origin_and_cleans_up,
//! a_remote_moved_by_someone_else_is_fetched_backed_out_onto_and_lands_again,
//! killed_between_the_steps_recovers_and_the_repeat_publishes_once,
//! with_no_landing_state_there_is_nothing_to_publish_and_the_remote_is_untouched,
//! publishing::a_remote_already_past_new_with_no_open_intent_is_published,
//! publishing::an_open_push_intent_after_the_push_is_closed_done_on_the_repeat,
//! publishing::an_open_push_intent_before_the_push_is_closed_failed_and_pushed_again and
//! publishing::an_open_push_intent_with_the_remote_past_new_is_closed_done.

use crate::landing::{publish_with_push, PublishAnswer, PublishError};
use crate::squash::LandRefs;
use gitio::Git;

/// Push on: the same as publish_with_push with push = true, answering Published { origin }, OriginMoved { found }
/// or NothingToPublish (or RemoteMissing as an error). The origin is never moved backward.
pub fn publish(git: &Git, refs: &LandRefs, actor: &str) -> Result<PublishAnswer, PublishError> {
    publish_with_push(git, refs, actor, true).map(|outcome| outcome.answer)
}
