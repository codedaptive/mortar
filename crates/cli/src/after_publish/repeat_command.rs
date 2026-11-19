//! repeat_command: the same after-publish command again, which a refused or failed folder's next ends with.

use super::AfterPublishArgs;

/// `mr after-publish <published> [--move <zone>]... --actor <name>`, the --moves in their order.
pub fn repeat_command(args: &AfterPublishArgs) -> String {
    let moves: String = args.moves.iter().map(|zone| format!(" --move {zone}")).collect();
    format!("mr after-publish {}{moves} --actor <name>", args.published)
}
