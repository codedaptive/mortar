//! Zone integration tests of publish (SPEC-land 04-publish.md §Tests): publishing (cases 1, 2, 4, 5 and 6) and kills
//! (case 3, with its worker), each case in its own throwaway local repository with a throwaway bare remote; and
//! state_display, PublishError's State text, which needs no repository.

#[path = "publish_zit/fixture.rs"]
mod fixture;
#[path = "publish_zit/kills.rs"]
mod kills;
#[path = "publish_zit/publishing.rs"]
mod publishing;
#[path = "publish_zit/state_display.rs"]
mod state_display;
