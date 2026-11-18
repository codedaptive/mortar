//! parse_install_args: the flags of `mr install`, their defaults, and each usage reason (spec 07 items 1 to 5).

use super::{parse_install_args, InstallOptions, SkillLocation};

fn parse(words: &[&str]) -> Result<InstallOptions, String> {
    parse_install_args(&words.iter().map(|word| word.to_string()).collect::<Vec<_>>())
}

fn reason(words: &[&str]) -> String {
    parse(words).expect_err("a usage error")
}

fn target_names(words: &[&str]) -> Vec<&'static str> {
    parse(words).expect("parses").targets.expect("targets given").iter().map(|h| h.name).collect()
}

#[test]
fn no_words_give_the_defaults() {
    assert_eq!(
        parse(&[]),
        Ok(InstallOptions {
            no_place: false,
            short_name: false,
            targets: None,
            yes: false,
            location: SkillLocation::Global,
        })
    );
}

#[test]
fn each_switch_sets_its_option() {
    let options = parse(&["--no-place", "--short-name", "-y"]).expect("parses");
    assert!(options.no_place && options.short_name && options.yes);
    assert!(parse(&["--yes"]).expect("parses").yes);
}

#[test]
fn target_takes_a_list_in_either_form_kept_once_in_table_order() {
    assert_eq!(target_names(&["--target", "cursor,claude"]), vec!["claude", "cursor"]);
    assert_eq!(target_names(&["--target=codex,codex,claude"]), vec!["claude", "codex"]);
}

#[test]
fn location_and_mode_take_their_words_in_either_form() {
    assert_eq!(parse(&["--location", "local"]).expect("parses").location, SkillLocation::Local);
    assert_eq!(parse(&["--location=global"]).expect("parses").location, SkillLocation::Global);
    assert!(parse(&["--mode", "skill"]).is_ok());
    assert!(parse(&["--mode=skill"]).is_ok());
}

#[test]
fn a_flag_twice_is_a_usage_error() {
    assert_eq!(reason(&["--no-place", "--no-place"]), "--no-place is given twice");
    assert_eq!(reason(&["--target", "claude", "--target=codex"]), "--target is given twice");
    assert_eq!(reason(&["-y", "--yes"]), "--yes is given twice");
    assert_eq!(reason(&["--mode=skill", "--mode", "skill"]), "--mode is given twice");
}

#[test]
fn a_missing_or_empty_value_is_a_usage_error() {
    assert_eq!(reason(&["--target"]), "--target needs a value");
    assert_eq!(reason(&["--target="]), "--target needs a value");
    assert_eq!(reason(&["--mode"]), "--mode needs a value");
    assert_eq!(reason(&["--location", ""]), "--location needs a value");
}

#[test]
fn a_bad_target_list_is_a_usage_error() {
    assert_eq!(reason(&["--target", "claude,,codex"]), "--target has an empty name");
    assert_eq!(reason(&["--target", "claude,"]), "--target has an empty name");
    assert_eq!(reason(&["--target", "vim"]), "unknown harness \"vim\": claude, codex or cursor");
}

#[test]
fn modes_other_than_skill_are_usage_errors() {
    assert_eq!(reason(&["--mode", "mcp"]), "Mortar offers --mode skill only; mcp and both are Tessera's");
    assert_eq!(reason(&["--mode=both"]), "Mortar offers --mode skill only; mcp and both are Tessera's");
    assert_eq!(reason(&["--mode", "plugin"]), "unknown mode \"plugin\": Mortar offers skill");
}

#[test]
fn an_unknown_location_is_a_usage_error() {
    assert_eq!(reason(&["--location", "remote"]), "unknown location \"remote\": global or local");
}

#[test]
fn any_other_word_is_a_usage_error() {
    assert_eq!(reason(&["claude"]), "unexpected word \"claude\" in install");
    assert_eq!(reason(&["--force"]), "unexpected word \"--force\" in install");
    assert_eq!(reason(&["--no-place=1"]), "unexpected word \"--no-place=1\" in install");
    assert_eq!(reason(&["-y", "extra"]), "unexpected word \"extra\" in install");
}
