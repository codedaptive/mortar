//! guard_decide: every case as data. Guarded refs are refs/heads/main (origin) and refs/heads/integration on remote
//! origin, whose push URLs are ORIGIN_SSH (the cases of push_url_cases also give ORIGIN_HTTPS); expected verdicts
//! are literals from guard_decide's contract (SPEC-land 06, S-60, S-100).

use gitio::Oid;
use oplog::RefMove;

use super::{guard_decide, GuardHook, GuardVerdict};
use crate::LandRefs;

const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const C: &str = "cccccccccccccccccccccccccccccccccccccccc";
const Z: &str = "0000000000000000000000000000000000000000";
const A64: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const B64: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const Z64: &str = "0000000000000000000000000000000000000000000000000000000000000000";

const ORIGIN_SSH: &str = "git@example.com:team/app.git";
const ORIGIN_HTTPS: &str = "https://example.com/team/app.git";
const FORK_URL: &str = "git@example.com:me/app.git";

const OWNER: &str = "the landing surface: mr land and mr publish";
const NEXT: &str = "bring your work in with `mr join`, or ask for a landing";

fn refs() -> LandRefs {
    LandRefs { origin_branch: "main".into(), integration_branch: "integration".into(), remote: "origin".into() }
}

fn mv(refname: &str, old: Option<&str>, new: &str) -> RefMove {
    RefMove { refname: refname.into(), old: old.map(|o| Oid::parse(o).unwrap()), new: Oid::parse(new).unwrap() }
}

fn tx(state: &str) -> GuardHook {
    GuardHook::ReferenceTransaction { state: state.into() }
}

/// A pre-push with arguments `remote` and `url`, origin's push URLs as the caller read them (None: unreadable).
fn push(remote: &str, url: &str, push_urls: Option<&[&str]>) -> GuardHook {
    let push_urls = push_urls.map(|urls| urls.iter().map(|u| u.to_string()).collect());
    GuardHook::PrePush { remote: remote.into(), url: url.into(), push_urls }
}

/// A push to origin by name, its push URLs read.
fn origin() -> GuardHook {
    push("origin", ORIGIN_SSH, Some(&[ORIGIN_SSH]))
}

/// A push to another remote by name, origin's push URLs read and not holding its URL.
fn fork() -> GuardHook {
    push("fork", FORK_URL, Some(&[ORIGIN_SSH]))
}

fn refuse(why: &str) -> GuardVerdict {
    GuardVerdict::Refuse { why: why.into(), owner: OWNER.into(), next: NEXT.into() }
}

struct Case {
    name: &'static str,
    hook: GuardHook,
    input: String,
    moves: Option<Vec<RefMove>>,
    want: GuardVerdict,
}

fn case(name: &'static str, hook: GuardHook, input: &str, moves: Option<Vec<RefMove>>, want: GuardVerdict) -> Case {
    Case { name, hook, input: input.into(), moves, want }
}

fn transaction_cases() -> Vec<Case> {
    let main_ab = format!("{A} {B} refs/heads/main\n");
    let integ_ab = format!("{A} {B} refs/heads/integration\n");
    vec![
        case("committed allowed", tx("committed"), &main_ab, None, GuardVerdict::Allow),
        case("aborted allowed", tx("aborted"), &main_ab, None, GuardVerdict::Allow),
        case("preparing allowed", tx("preparing"), &main_ab, None, GuardVerdict::Allow),
        case("non-prepared not read", tx("committed"), "garbage", None, GuardVerdict::Allow),
        case("unguarded ref no intent", tx("prepared"), &format!("{A} {B} refs/heads/feature\n"), None,
            GuardVerdict::Allow),
        case("origin no intent", tx("prepared"), &main_ab, None, refuse("only Mortar moves refs/heads/main")),
        case("integration no intent", tx("prepared"), &integ_ab, None,
            refuse("only Mortar moves refs/heads/integration")),
        case("empty intent", tx("prepared"), &main_ab, Some(vec![]), refuse("only Mortar moves refs/heads/main")),
        case("wrong old", tx("prepared"), &main_ab, Some(vec![mv("refs/heads/main", Some(C), B)]),
            refuse("only Mortar moves refs/heads/main")),
        case("wrong new", tx("prepared"), &main_ab, Some(vec![mv("refs/heads/main", Some(A), C)]),
            refuse("only Mortar moves refs/heads/main")),
        case("other ref", tx("prepared"), &main_ab, Some(vec![mv("refs/heads/integration", Some(A), B)]),
            refuse("only Mortar moves refs/heads/main")),
        case("exact match", tx("prepared"), &main_ab, Some(vec![mv("refs/heads/main", Some(A), B)]),
            GuardVerdict::Allow),
        case("zero old matched by None", tx("prepared"), &format!("{Z} {B} refs/heads/main"),
            Some(vec![mv("refs/heads/main", None, B)]), GuardVerdict::Allow),
        case("zero old against Some", tx("prepared"), &format!("{Z} {B} refs/heads/main"),
            Some(vec![mv("refs/heads/main", Some(A), B)]), refuse("only Mortar moves refs/heads/main")),
        case("real old against None", tx("prepared"), &main_ab, Some(vec![mv("refs/heads/main", None, B)]),
            refuse("only Mortar moves refs/heads/main")),
        case("deletion with zero new", tx("prepared"), &format!("{A} {Z} refs/heads/main"),
            Some(vec![mv("refs/heads/main", Some(A), Z)]), GuardVerdict::Allow),
        case("deletion with real new", tx("prepared"), &format!("{A} {Z} refs/heads/main"),
            Some(vec![mv("refs/heads/main", Some(A), B)]), refuse("only Mortar moves refs/heads/main")),
        case("unguarded then guarded", tx("prepared"),
            &format!("{A} {B} refs/heads/feature\n{A} {B} refs/heads/integration\n"),
            Some(vec![mv("refs/heads/main", Some(A), B)]), refuse("only Mortar moves refs/heads/integration")),
        case("two guarded both matched", tx("prepared"),
            &format!("{A} {B} refs/heads/main\n{B} {C} refs/heads/integration\n"),
            Some(vec![mv("refs/heads/integration", Some(B), C), mv("refs/heads/main", Some(A), B)]),
            GuardVerdict::Allow),
        case("sha256 match", tx("prepared"), &format!("{A64} {B64} refs/heads/main\n"),
            Some(vec![mv("refs/heads/main", Some(A64), B64)]), GuardVerdict::Allow),
        case("sha256 zero old", tx("prepared"), &format!("{Z64} {B64} refs/heads/main\n"),
            Some(vec![mv("refs/heads/main", None, B64)]), GuardVerdict::Allow),
        case("sha256 unmatched", tx("prepared"), &format!("{A64} {B64} refs/heads/main\n"), None,
            refuse("only Mortar moves refs/heads/main")),
        case("empty input", tx("prepared"), "", None, GuardVerdict::Allow),
        case("two fields", tx("prepared"), &format!("{A} {B}\n"), None,
            refuse(&format!("cannot read the reference-transaction hook's input line \"{A} {B}\""))),
        case("uppercase hex", tx("prepared"), "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb refs/heads/x",
            None, refuse("cannot read the reference-transaction hook's input line \"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb refs/heads/x\"")),
        case("short id", tx("prepared"), "abc bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb refs/heads/x", None,
            refuse("cannot read the reference-transaction hook's input line \"abc bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb refs/heads/x\"")),
    ]
}

fn push_cases() -> Vec<Case> {
    let main_line = format!("refs/heads/main {B} refs/heads/main {A}\n");
    vec![
        case("other remote", fork(), &main_line, None, GuardVerdict::Allow),
        case("other remote not read", fork(), "garbage", None, GuardVerdict::Allow),
        case("guarded matched", origin(), &main_line, Some(vec![mv("refs/heads/main", Some(A), B)]),
            GuardVerdict::Allow),
        case("guarded unmatched", origin(), &main_line, None,
            refuse("only Mortar pushes refs/heads/main to origin")),
        case("guarded wrong old", origin(), &main_line, Some(vec![mv("refs/heads/main", Some(C), B)]),
            refuse("only Mortar pushes refs/heads/main to origin")),
        case("integration new on remote", origin(), &format!("refs/heads/x {B} refs/heads/integration {Z}"),
            Some(vec![mv("refs/heads/integration", None, B)]), GuardVerdict::Allow),
        case("unguarded branch", origin(), &format!("refs/heads/x {B} refs/heads/x {A}"), None,
            GuardVerdict::Allow),
        case("delete matched", origin(), &format!("(delete) {Z} refs/heads/main {A}"),
            Some(vec![mv("refs/heads/main", Some(A), Z)]), GuardVerdict::Allow),
        case("delete unmatched", origin(), &format!("(delete) {Z} refs/heads/main {A}"), None,
            refuse("only Mortar pushes refs/heads/main to origin")),
        case("push empty input", origin(), "", None, GuardVerdict::Allow),
        case("push unreadable", origin(), &format!("refs/heads/main {B} refs/heads/main"), None,
            refuse(&format!("cannot read the pre-push hook's input line \"refs/heads/main {B} refs/heads/main\""))),
    ]
}

/// S-100: the push is to origin when arg 1 is its name or the URL is exactly one of its push URLs; when its push
/// URLs could not be read and arg 1 is another name, the whole push is refused unread.
fn push_url_cases() -> Vec<Case> {
    let main_line = format!("refs/heads/main {B} refs/heads/main {A}\n");
    let main_ab = || Some(vec![mv("refs/heads/main", Some(A), B)]);
    let both: &[&str] = &[ORIGIN_SSH, ORIGIN_HTTPS];
    let unmatched = refuse("only Mortar pushes refs/heads/main to origin");
    let unreadable_urls = refuse("cannot read origin's push URLs, so the guard cannot tell whether the push to \
        git@example.com:me/app.git is to origin");
    vec![
        case("by url matched", push(ORIGIN_SSH, ORIGIN_SSH, Some(&[ORIGIN_SSH])), &main_line, main_ab(),
            GuardVerdict::Allow),
        case("by url unmatched", push(ORIGIN_SSH, ORIGIN_SSH, Some(&[ORIGIN_SSH])), &main_line, None,
            unmatched.clone()),
        case("by url second of several", push(ORIGIN_HTTPS, ORIGIN_HTTPS, Some(both)), &main_line, None,
            unmatched.clone()),
        case("url not among push urls", push(FORK_URL, FORK_URL, Some(both)), &main_line, None, GuardVerdict::Allow),
        case("named fork not among push urls", push("fork", FORK_URL, Some(both)), &main_line, None,
            GuardVerdict::Allow),
        case("trailing slash not matched", push("git@example.com:team/app.git/", "git@example.com:team/app.git/",
            Some(&[ORIGIN_SSH])), &main_line, None, GuardVerdict::Allow),
        case(".git removed not matched", push("git@example.com:team/app", "git@example.com:team/app",
            Some(&[ORIGIN_SSH])), &main_line, None, GuardVerdict::Allow),
        case(".git added not matched", push("https://example.com/team/app.git", "https://example.com/team/app.git",
            Some(&["https://example.com/team/app"])), &main_line, None, GuardVerdict::Allow),
        case("case differs not matched", push("git@example.com:Team/App.git", "git@example.com:Team/App.git",
            Some(&[ORIGIN_SSH])), &main_line, None, GuardVerdict::Allow),
        case("empty push urls other name", push("fork", FORK_URL, Some(&[])), &main_line, None, GuardVerdict::Allow),
        case("name is enough", push("origin", "https://elsewhere.example/app.git", Some(&[ORIGIN_SSH])), &main_line,
            None, unmatched.clone()),
        case("unread urls by name", push("origin", ORIGIN_SSH, None), &main_line, None, unmatched),
        case("unread urls by name unguarded", push("origin", ORIGIN_SSH, None),
            &format!("refs/heads/x {B} refs/heads/x {A}"), None, GuardVerdict::Allow),
        case("unread urls other name unmatched", push("fork", FORK_URL, None), &main_line, None,
            unreadable_urls.clone()),
        case("unread urls other name matched", push("fork", FORK_URL, None), &main_line, main_ab(),
            unreadable_urls.clone()),
        case("unread urls other name unguarded", push("fork", FORK_URL, None),
            &format!("refs/heads/x {B} refs/heads/x {A}"), None, unreadable_urls.clone()),
        case("unread urls other name unreadable", push("fork", FORK_URL, None), "garbage", None,
            unreadable_urls.clone()),
        case("unread urls other name empty input", push("fork", FORK_URL, None), "", None, unreadable_urls),
    ]
}

#[test]
fn guard_decide_cases() {
    for c in transaction_cases().into_iter().chain(push_cases()).chain(push_url_cases()) {
        let got = guard_decide(&c.hook, &c.input, &refs(), c.moves.as_deref());
        assert_eq!(got, c.want, "case {:?}", c.name);
    }
}
