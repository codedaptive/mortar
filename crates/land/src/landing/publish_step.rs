//! PublishStep: what publish does next, as decide_publish chooses it. A bare type; decide_publish_tests exercises
//! each variant.

use gitio::Oid;

/// Nothing: no landing waits. Push: step 1, push the landing to the remote. Advance: the remote has it (with push
/// off: step 2 without the remote); step 2, move the local origin. Cleanup: the local origin has it; remove the
/// squash ref and the landing state. FetchOrigin: someone else moved the remote; fetch it and back the landing out
/// onto it. BackOut: the local origin already moved to `onto` for that (with push off: the local origin was found
/// at `onto`); finish the back-out onto it. RemoteMissing: the remote has no origin branch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum PublishStep {
    Nothing,
    Push,
    Advance,
    Cleanup,
    FetchOrigin,
    BackOut { onto: Oid },
    RemoteMissing,
}
