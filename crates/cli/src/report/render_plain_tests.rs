//! render_plain: the lines, then owner when there is one, and always `next: ...` last.

use super::{render_plain, ExitCode, OutValue, Report};

fn report(lines: &[&str], owner: Option<&str>, next: &str) -> Report {
    Report {
        verb: "init".to_string(),
        exit: ExitCode::Done,
        outcome: "written".to_string(),
        lines: lines.iter().map(|l| l.to_string()).collect(),
        owner: owner.map(str::to_string),
        next: next.to_string(),
        fields: vec![("config_path".to_string(), OutValue::Text("/r/.git/mortar/config.json".to_string()))],
    }
}

#[test]
fn lines_then_next_and_no_fields() {
    assert_eq!(
        render_plain(&report(&["wrote /c.json", "zones: a"], None, "mr folder zone a p")),
        "wrote /c.json\nzones: a\nnext: mr folder zone a p\n"
    );
}

#[test]
fn a_refusal_shows_its_owner_before_next() {
    let mut refused = report(&["config differs in: remote"], Some("the existing config at /c.json"), "mr init");
    refused.exit = ExitCode::Refused;
    refused.outcome = "refused".to_string();
    assert_eq!(
        render_plain(&refused),
        "config differs in: remote\nowner: the existing config at /c.json\nnext: mr init\n"
    );
}

#[test]
fn no_lines_is_only_the_next_line() {
    assert_eq!(render_plain(&report(&[], None, "repeat the same command")), "next: repeat the same command\n");
}

#[test]
fn the_last_line_is_always_next() {
    for owner in [None, Some("pid 7 on h")] {
        let text = render_plain(&report(&["a", "b"], owner, "do it"));
        assert_eq!(text.lines().last(), Some("next: do it"));
    }
}
