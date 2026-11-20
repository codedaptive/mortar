//! Zone integration tests of publish: publishing (the remote then the local origin moved, a remote moved by someone
//! else, nothing to publish, refused pushes, a push intent left open settled on the repeat, a remote already past the
//! squash with no intent) and kills (a publish killed between its two steps, with its worker), each case in its own
//! throwaway local repository with a throwaway bare remote.

#[path = "publish_zit/fixture.rs"]
mod fixture;
#[path = "publish_zit/kills.rs"]
mod kills;
#[path = "publish_zit/publishing.rs"]
mod publishing;
