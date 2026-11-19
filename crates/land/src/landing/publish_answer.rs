//! PublishAnswer: what publish did. A bare type; the publish_zit tests
//! publish_moves_the_remote_then_the_local_origin_and_cleans_up (Published),
//! a_remote_moved_by_someone_else_is_fetched_backed_out_onto_and_lands_again (OriginMoved) and
//! with_no_landing_state_there_is_nothing_to_publish_and_the_remote_is_untouched (NothingToPublish) exercise each
//! variant.

use gitio::Oid;

/// Published: the remote and the local origin branch are at `origin`, the landed commit; the landing state and the
/// zone's squash ref are removed. OriginMoved: someone else moved the remote's origin branch, so the local origin
/// was fast-forwarded to `found`, integration put on it and the landing state removed; the squash and its ref are
/// kept, and the next step is to land again. NothingToPublish: no landing state, so no landing is waiting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PublishAnswer {
    Published { origin: Oid },
    OriginMoved { found: Oid },
    NothingToPublish,
}
