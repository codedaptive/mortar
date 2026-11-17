//! repeat_command: the same zones, published first and each --move in order, with --actor <name>.

use super::repeat_command;
use crate::after_publish::AfterPublishArgs;

fn args(published: &str, moves: &[&str]) -> AfterPublishArgs {
    AfterPublishArgs { published: published.to_string(), moves: moves.iter().map(ToString::to_string).collect() }
}

#[test]
fn no_moves_names_the_published_zone_only() {
    assert_eq!(repeat_command(&args("cli", &[])), "mr after-publish cli --actor <name>");
}

#[test]
fn one_move_follows_the_published_zone() {
    assert_eq!(repeat_command(&args("cli", &["land"])), "mr after-publish cli --move land --actor <name>");
}

#[test]
fn two_moves_keep_their_order() {
    assert_eq!(
        repeat_command(&args("land", &["join", "cli"])),
        "mr after-publish land --move join --move cli --actor <name>"
    );
}
