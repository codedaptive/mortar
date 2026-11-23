//! Zone integration tests of publish: publishing (the remote then the local origin moved, a remote moved by someone
//! else, nothing to publish, refused pushes, a push intent left open settled on the repeat, a remote already past the
//! squash with no intent), kills (a publish killed between its two steps, with its worker), and no_push (push off,
//! cleanup after interruption, moved local origin, and push-on outcomes), and repeats (mixed push settings across
//! interruptions), and status (the landing status read after a publish killed inside its push, and with no publish
//! in progress), each in a throwaway local and bare remote.

#[path = "publish_zit/fixture.rs"]
mod fixture;
#[path = "publish_zit/kills.rs"]
mod kills;
#[path = "publish_zit/no_push.rs"]
mod no_push;
#[path = "publish_zit/publishing.rs"]
mod publishing;
#[path = "publish_zit/repeats.rs"]
mod repeats;
#[path = "publish_zit/status.rs"]
mod status;
