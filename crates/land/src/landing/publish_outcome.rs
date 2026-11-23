//! PublishOutcome: what publish_with_push did and whether it pushed. A bare type with no sibling test; the
//! publish_zit tests no_push::push_off_publishes_locally_without_reading_the_remote,
//! no_push::push_off_repeat_after_origin_advanced_only_cleans_up,
//! no_push::push_off_moved_local_origin_backs_out_without_fetching, no_push::push_on_published_reports_pushed and
//! no_push::push_on_moved_remote_reports_not_pushed check it.

use crate::landing::PublishAnswer;

/// `answer`: what publish did. `pushed`: true only when the answer is Published and this publish's push intent
/// closed done, on either setting, so the published commit is on the remote; false otherwise.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublishOutcome {
    pub answer: PublishAnswer,
    pub pushed: bool,
}
