//! CHANGE_CONFLICT_NEXT: the next step when a change's squash conflicts with integration, shared by would-merge's
//! No, would-merge --with's Conflicted and land's Conflicted.
//! Its sibling change_conflict_next_tests pins its text.

/// Fix the conflict in the change's zone through a request, then make the zone's squash again.
pub const CHANGE_CONFLICT_NEXT: &str = "fix the conflict in the change's zone through a request (mr folder request \
                                        <zone> <request> <path> --paths <path>... --actor <name>), then mr \
                                        squash <zone> --actor <name>";
