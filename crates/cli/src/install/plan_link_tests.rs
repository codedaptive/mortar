//! Tests of plan_link: the target, the clash, what stands at `~/.local/bin/mr`, and the refusal (spec 07 item 3).

use std::path::PathBuf;

use super::{plan_link, BinaryAt, InstallFacts, InstallItem, InstallPaths, InstallStep, LinkAt, LinkPlan};
use crate::report::ExitCode;

fn paths() -> InstallPaths {
    InstallPaths {
        bin_dir: PathBuf::from("/h/.local/bin"),
        mortar: PathBuf::from("/h/.local/bin/mortar"),
        mr: PathBuf::from("/h/.local/bin/mr"),
        state_dir: PathBuf::from("/h/.local/state/mortar"),
        record: PathBuf::from("/h/.local/state/mortar/install.json"),
    }
}

fn facts(link_at: LinkAt) -> InstallFacts {
    InstallFacts {
        this: Some(PathBuf::from("/opt/homebrew/Cellar/mortar/1/bin/mortar")),
        this_hash: None,
        binary_at: BinaryAt::Absent,
        link_at,
        first_mr: None,
        first_mortar: Some((PathBuf::from("/opt/homebrew/bin/mortar"), true)),
        first_mr_at: None,
        bin_at: Some(0),
        exe_dir: Some(PathBuf::from("/opt/homebrew/bin")),
        detected: Vec::new(),
        skills: Vec::new(),
        record: Vec::new(),
    }
}

fn line(action: &str, note: &str) -> InstallItem {
    InstallItem { action: action.to_string(), path: PathBuf::from("/h/.local/bin/mr"), note: note.to_string() }
}

fn link_to(target: &str) -> Option<InstallStep> {
    Some(InstallStep::Link { at: PathBuf::from("/h/.local/bin/mr"), target: PathBuf::from(target) })
}

fn placed_link(at: LinkAt) -> LinkPlan {
    plan_link(&facts(at), &paths(), true, false).unwrap()
}

#[test]
fn placed_links_to_the_placed_binary() {
    let plan = placed_link(LinkAt::Absent);
    let want = LinkPlan {
        step: link_to("/h/.local/bin/mortar"),
        item: line("linked", "-> /h/.local/bin/mortar"),
        present: true,
        next: None,
    };
    assert_eq!(plan, want);
}

#[test]
fn not_placed_links_to_the_long_name_where_it_stands_on_path() {
    let plan = plan_link(&facts(LinkAt::Absent), &paths(), false, false).unwrap();
    assert_eq!(plan.step, link_to("/opt/homebrew/bin/mortar"));
    assert_eq!(plan.item, line("linked", "-> /opt/homebrew/bin/mortar"));
    assert!(plan.present);
}

#[test]
fn not_placed_and_no_mortar_on_path_is_this_binary_skips_the_link() {
    for first in [None, Some((PathBuf::from("/usr/bin/mortar"), false))] {
        let mut facts = facts(LinkAt::Foreign);
        facts.first_mortar = first;
        let plan = plan_link(&facts, &paths(), false, true).unwrap();
        let want = LinkPlan {
            step: None,
            item: line("link skipped", "no mortar on PATH is this binary"),
            present: false,
            next: Some("add /opt/homebrew/bin to PATH".to_string()),
        };
        assert_eq!(plan, want);
    }
}

#[test]
fn no_target_and_no_binary_folder_names_the_package_bin_folder() {
    let mut facts = facts(LinkAt::Absent);
    facts.first_mortar = None;
    facts.exe_dir = None;
    let plan = plan_link(&facts, &paths(), false, false).unwrap();
    assert_eq!(plan.next.as_deref(), Some("add the package's bin folder to PATH"));
}

#[test]
fn a_foreign_first_mr_skips_the_link_even_over_a_foreign_file() {
    let mut facts = facts(LinkAt::Foreign);
    facts.first_mr = Some((PathBuf::from("/usr/bin/mr"), false));
    let plan = plan_link(&facts, &paths(), true, false).unwrap();
    let note = "the first mr on PATH is /usr/bin/mr, another program: run Mortar as mortar, or link anyway with \
                --short-name";
    assert_eq!(plan, LinkPlan { step: None, item: line("link skipped", note), present: false, next: None });
}

#[test]
fn the_clash_skip_keeps_an_earlier_link_present() {
    let mut facts = facts(LinkAt::Link { target: PathBuf::from("/h/.local/bin/mortar"), mortar: true });
    facts.first_mr = Some((PathBuf::from("/usr/bin/mr"), false));
    let plan = plan_link(&facts, &paths(), true, false).unwrap();
    assert_eq!(plan.step, None);
    assert!(plan.present);
}

#[test]
fn short_name_links_behind_a_foreign_first_mr() {
    let mut facts = facts(LinkAt::Absent);
    facts.first_mr = Some((PathBuf::from("/usr/bin/mr"), false));
    let plan = plan_link(&facts, &paths(), true, true).unwrap();
    assert_eq!(plan.step, link_to("/h/.local/bin/mortar"));
    assert!(plan.present);
}

#[test]
fn a_mortar_first_mr_is_no_clash() {
    let mut facts = facts(LinkAt::Absent);
    facts.first_mr = Some((PathBuf::from("/usr/bin/mr"), true));
    assert_eq!(plan_link(&facts, &paths(), true, false).unwrap().step, link_to("/h/.local/bin/mortar"));
}

#[test]
fn the_same_link_is_unchanged() {
    let plan = placed_link(LinkAt::Link { target: PathBuf::from("/h/.local/bin/mortar"), mortar: false });
    let want = LinkPlan { step: None, item: line("unchanged", "-> /h/.local/bin/mortar"), present: true, next: None };
    assert_eq!(plan, want);
}

#[test]
fn a_mortar_link_elsewhere_or_a_mortar_file_is_relinked() {
    for at in [LinkAt::Link { target: PathBuf::from("/old/mortar"), mortar: true }, LinkAt::Mortar] {
        let plan = placed_link(at);
        assert_eq!(plan.step, link_to("/h/.local/bin/mortar"));
        assert_eq!(plan.item, line("linked", "-> /h/.local/bin/mortar"));
    }
}

#[test]
fn a_foreign_link_or_file_is_refused_when_linking() {
    for at in [LinkAt::Link { target: PathBuf::from("/usr/bin/other"), mortar: false }, LinkAt::Foreign] {
        let report = plan_link(&facts(at), &paths(), true, true).unwrap_err();
        assert_eq!(report.exit, ExitCode::Refused);
        assert_eq!(report.outcome, "refused");
        assert_eq!(report.lines, ["/h/.local/bin/mr is another program, not Mortar's link", "nothing changed"]);
        assert_eq!(report.owner.as_deref(), Some("whoever put /h/.local/bin/mr there"));
        assert_eq!(report.next, "move /h/.local/bin/mr aside, then install again");
    }
}
