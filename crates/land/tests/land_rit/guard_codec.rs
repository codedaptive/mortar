//! Twelve owed guard codec seams, through PUBLIC guard_install/read_guard_config and real persisted bytes.
//! Land -> Oplog JSON strings; the private parser/encoder are neither copied nor included.
use gitio::Git;
use land::{GuardError, LandRefs};
use std::{fs, path::PathBuf};
use testkit::TestRepo;

const PLAIN: &str = "{\"origin_branch\":\"main\",\"integration_branch\":\"integration\",\"remote\":\"origin\"}";
fn refs(origin: &str, integration: &str, remote: &str) -> LandRefs {
    LandRefs { origin_branch: origin.into(), integration_branch: integration.into(), remote: remote.into() }
}
fn main_refs() -> LandRefs { refs("main", "integration", "origin") }
struct Guard { git: Git, path: PathBuf, _repo: TestRepo }
impl Guard {
    fn new() -> Self {
        let repo = TestRepo::new().unwrap(); let mut git = Git::new(&repo.path);
        git.env = vec![("GIT_CONFIG_GLOBAL".into(), "/dev/null".into()), ("GIT_CONFIG_NOSYSTEM".into(), "1".into()), ("LC_ALL".into(), "C".into())];
        let path = git.common_dir().unwrap().join("mortar/guard.json");
        Self { git, path, _repo: repo }
    }
    fn write(&self, text: &str) {
        fs::create_dir_all(self.path.parent().unwrap()).unwrap(); fs::write(&self.path, text).unwrap();
    }
    fn read(&self) -> LandRefs { land::read_guard_config(&self.git).unwrap().unwrap() }
    fn refused(&self, text: &str) -> String {
        self.write(text);
        match land::read_guard_config(&self.git) {
            Err(GuardError::BadConfig { path, why }) => { assert_eq!(path, self.path); why }
            other => panic!("expected BadConfig for {text:?}, got {other:?}"),
        }
    }
}

#[test]
fn rit_guard_config_plain_names_have_the_literal_key_order_and_one_final_newline() {
    let f = Guard::new(); land::guard_install(&f.git, &main_refs()).unwrap();
    assert_eq!(fs::read(&f.path).unwrap(), b"{\"origin_branch\":\"main\",\"integration_branch\":\"integration\",\"remote\":\"origin\"}\n");
}

#[test]
fn rit_guard_config_names_needing_escapes_stay_on_one_line() {
    let f = Guard::new(); land::guard_install(&f.git, &refs("ma\"in", "integ\\ration", "or\nigin")).unwrap();
    assert_eq!(fs::read(&f.path).unwrap(), b"{\"origin_branch\":\"ma\\\"in\",\"integration_branch\":\"integ\\\\ration\",\"remote\":\"or\\nigin\"}\n");
}

#[test]
fn rit_guard_config_round_trip_over_many_names() {
    let f = Guard::new();
    let names = ["main", "", "a\"b", "back\\slash", "\"\\\"", "line\nbreak", "tab\there", "cr\rlf", "\u{0}", "\u{1f}",
        "\u{8}\u{c}", "\u{7f}", "café", "分支", "emoji 🦘", "/ slash", "{\"key\":\"x\"}", "  spaced  "];
    for (i, a) in names.iter().enumerate() {
        let original = refs(a, names[(i + 3) % names.len()], names[(i + 7) % names.len()]);
        land::guard_install(&f.git, &original).unwrap(); assert_eq!(f.read(), original);
    }
}

#[test]
fn rit_guard_config_keys_in_any_order_with_whitespace() {
    let f = Guard::new(); f.write(" \r\n{ \"remote\" :\t\"origin\" ,\n\"integration_branch\":\"integration\",\"origin_branch\" : \"main\" }\n");
    assert_eq!(f.read(), main_refs());
}

#[test]
fn rit_guard_config_no_trailing_newline_is_fine() {
    let f = Guard::new(); f.write(PLAIN); assert_eq!(f.read(), main_refs());
}

#[test]
fn rit_guard_config_escapes_written_another_way_still_read() {
    let f = Guard::new(); f.write(r#"{"origin_branch":"main","integration_branch":"integration","remote":"o\/r"}"#);
    assert_eq!(f.read(), refs("main", "integration", "o/r"));
}

#[test]
fn rit_guard_config_a_missing_key_is_refused_naming_it() {
    let f = Guard::new();
    assert_eq!(f.refused(r#"{"remote":"origin","origin_branch":"main"}"#), "key \"integration_branch\" is missing");
    assert_eq!(f.refused("{\"remote\":\"origin\"}\n"), "key \"origin_branch\" is missing");
    assert_eq!(f.refused("{}"), "key \"origin_branch\" is missing");
    assert_eq!(f.refused(r#"{"origin_branch":"main","integration_branch":"integration"}"#), "key \"remote\" is missing");
}

#[test]
fn rit_guard_config_a_repeated_key_is_refused_naming_it() {
    let f = Guard::new();
    assert_eq!(f.refused(r#"{"origin_branch":"main","integration_branch":"integration","remote":"origin","remote":"o"}"#), "key \"remote\" appears twice");
}

#[test]
fn rit_guard_config_an_unknown_key_is_refused_naming_it() {
    let f = Guard::new();
    assert_eq!(f.refused(r#"{"origin_branch":"main","integration_branch":"integration","remote":"origin","extra":"x"}"#), "unknown key \"extra\"");
}

#[test]
fn rit_guard_config_a_non_string_value_is_refused() {
    let f = Guard::new();
    for value in ["1", "null", "true", "[\"main\"]", "{}"] {
        let text = format!("{{\"origin_branch\":{value},\"integration_branch\":\"integration\",\"remote\":\"origin\"}}");
        assert_eq!(f.refused(&text), "the value of \"origin_branch\" is not a string: no string at byte 17");
    }
}

#[test]
fn rit_guard_config_text_after_the_object_is_refused() {
    let f = Guard::new();
    for tail in ["x", "\n{}\n"] { assert!(f.refused(&format!("{PLAIN}{tail}")).starts_with("text after the object at byte ")); }
}

#[test]
fn rit_guard_config_a_broken_object_is_refused() {
    let f = Guard::new();
    for (text, why) in [
        ("{", "a key at byte 1: no string at byte 1"),
        ("{\"origin_branch\"", "no ':' after key \"origin_branch\" at byte 16"),
        ("{\"origin_branch\":", "the value of \"origin_branch\" is not a string: no string at byte 17"),
        ("{\"origin_branch\":\"main", "the value of \"origin_branch\" is not a string: the text ends inside a string"),
        ("{\"origin_branch\" \"main\"}", "no ':' after key \"origin_branch\" at byte 17"),
        ("{\"origin_branch\":\"main\" \"remote\":\"origin\"}", "no ',' or '}' after a member at byte 24"),
        ("{\"origin_branch\":\"main\",}", "a key at byte 24: no string at byte 24"),
        ("{origin_branch:\"main\"}", "a key at byte 1: no string at byte 1"),
        ("{\"origin_branch\":\"ma\nin\",\"integration_branch\":\"integration\",\"remote\":\"origin\"}",
            "the value of \"origin_branch\" is not a string: raw control char U+000A in a string at byte 20; it must be escaped"),
    ] {
        assert_eq!(f.refused(text), why, "{text:?}");
    }
}
