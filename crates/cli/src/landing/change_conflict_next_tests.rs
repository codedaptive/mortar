//! CHANGE_CONFLICT_NEXT: the exact next step of a change whose squash conflicts with integration, naming commands
//! the binary runs now.

use super::CHANGE_CONFLICT_NEXT;

#[test]
fn the_next_step_is_the_contract_text() {
    assert_eq!(
        CHANGE_CONFLICT_NEXT,
        "fix the conflict in the change's zone through a request (mortar folder request <zone> <request> <path> \
         --paths <path>... --actor <name>), then mortar squash <zone> --actor <name>"
    );
}

#[test]
fn it_names_the_request_folder_and_the_squash_commands() {
    assert!(CHANGE_CONFLICT_NEXT.contains("mortar folder request <zone> <request> <path> --paths <path>..."));
    assert!(CHANGE_CONFLICT_NEXT.ends_with("then mortar squash <zone> --actor <name>"));
}
