//! Tests of skill_command: every case of the rule (spec 07 item 6, S-177).

use super::skill_command;

#[test]
fn link_present_and_first_on_path_names_mr() {
    assert_eq!(skill_command(true, Some(0), None), "mr");
    assert_eq!(skill_command(true, Some(2), Some((3, false))), "mr");
}

#[test]
fn an_mr_at_the_link_folder_itself_is_the_link_now() {
    assert_eq!(skill_command(true, Some(1), Some((1, false))), "mr");
}

#[test]
fn a_foreign_mr_earlier_on_path_names_mortar_even_with_the_link() {
    assert_eq!(skill_command(true, Some(1), Some((0, false))), "mortar");
}

#[test]
fn a_mortar_mr_earlier_on_path_names_mr() {
    assert_eq!(skill_command(true, Some(1), Some((0, true))), "mr");
    assert_eq!(skill_command(false, Some(1), Some((0, true))), "mr");
}

#[test]
fn local_bin_not_on_path_falls_back_to_the_first_mr() {
    assert_eq!(skill_command(true, None, None), "mortar");
    assert_eq!(skill_command(true, None, Some((0, false))), "mortar");
    assert_eq!(skill_command(true, None, Some((0, true))), "mr");
}

#[test]
fn no_link_falls_back_to_the_first_mr() {
    assert_eq!(skill_command(false, Some(0), None), "mortar");
    assert_eq!(skill_command(false, Some(0), Some((0, false))), "mortar");
    assert_eq!(skill_command(false, Some(0), Some((2, true))), "mr");
}
