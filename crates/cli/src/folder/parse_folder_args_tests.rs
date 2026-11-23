//! parse_folder_args: each sub-verb's exact words and flags into a FolderCommand, and each word it refuses.

use std::path::PathBuf;

use super::{parse_folder_args, FolderCommand};

fn parse(words: &[&str]) -> Result<FolderCommand, String> {
    parse_folder_args(&words.iter().map(ToString::to_string).collect::<Vec<_>>())
}

fn zone(start: Option<&str>) -> FolderCommand {
    FolderCommand::Zone { zone: "core".to_string(), path: PathBuf::from("/w/core"), start: start.map(str::to_string) }
}

#[test]
fn zone_takes_its_zone_and_path() {
    assert_eq!(parse(&["zone", "core", "/w/core"]), Ok(zone(None)));
}

#[test]
fn zone_takes_start_before_after_or_joined() {
    assert_eq!(parse(&["zone", "core", "/w/core", "--start", "main"]), Ok(zone(Some("main"))));
    assert_eq!(parse(&["zone", "--start", "main", "core", "/w/core"]), Ok(zone(Some("main"))));
    assert_eq!(parse(&["zone", "core", "--start=v1.0", "/w/core"]), Ok(zone(Some("v1.0"))));
}

#[test]
fn zone_refuses_a_start_given_twice_missing_empty_or_dashed() {
    assert_eq!(
        parse(&["zone", "core", "/w/core", "--start", "a", "--start=b"]),
        Err("--start is given twice".to_string())
    );
    assert_eq!(parse(&["zone", "core", "/w/core", "--start"]), Err("--start needs a commit".to_string()));
    assert_eq!(parse(&["zone", "core", "/w/core", "--start="]), Err("--start needs a commit".to_string()));
    assert_eq!(
        parse(&["zone", "core", "/w/core", "--start", "--upload-pack=x"]),
        Err("--start needs a commit, not \"--upload-pack=x\"".to_string())
    );
}

#[test]
fn zone_refuses_a_missing_or_extra_word_and_any_other_flag() {
    assert_eq!(parse(&["zone", "core"]), Err("folder zone needs <zone> <path>".to_string()));
    assert_eq!(parse(&["zone", "core", "/w/core", "x"]), Err("unexpected word \"x\" in folder zone".to_string()));
    assert_eq!(
        parse(&["zone", "core", "/w/core", "--paths", "src"]),
        Err("unknown folder zone argument \"--paths\"".to_string())
    );
}

#[test]
fn request_takes_its_names_path_and_every_word_after_paths() {
    assert_eq!(
        parse(&["request", "core", "fix-1", "/w/fix", "--paths", "src/a", "docs"]),
        Ok(FolderCommand::Request {
            zone: "core".to_string(),
            request: "fix-1".to_string(),
            path: PathBuf::from("/w/fix"),
            paths: vec!["src/a".to_string(), "docs".to_string()],
            partners: Vec::new(),
        })
    );
}

fn request(paths: &[&str], partners: &[&str]) -> FolderCommand {
    FolderCommand::Request {
        zone: "core".to_string(),
        request: "fix-1".to_string(),
        path: PathBuf::from("/w/fix"),
        paths: paths.iter().map(ToString::to_string).collect(),
        partners: partners.iter().map(ToString::to_string).collect(),
    }
}

#[test]
fn request_takes_with_partners_in_order_before_or_after_paths() {
    let rows: Vec<(&[&str], Result<FolderCommand, String>)> = vec![
        (
            &["request", "core", "fix-1", "/w/fix", "--paths", "src/a", "--with", "web"],
            Ok(request(&["src/a"], &["web"])),
        ),
        (
            &["request", "core", "fix-1", "/w/fix", "--with=web", "--paths", "src/a", "docs"],
            Ok(request(&["src/a", "docs"], &["web"])),
        ),
        (
            &["request", "core", "fix-1", "/w/fix", "--with", "web", "--paths", "src/a", "--with=ui", "--with", "db"],
            Ok(request(&["src/a"], &["web", "ui", "db"])),
        ),
    ];
    for (words, expected) in rows {
        assert_eq!(parse(words), expected, "{words:?}");
    }
}

#[test]
fn request_refuses_a_with_missing_empty_dashed_repeated_or_its_own_zone() {
    let rows: Vec<(&[&str], &str)> = vec![
        (&["request", "core", "fix-1", "/w/fix", "--paths", "a", "--with"], "--with needs a zone"),
        (&["request", "core", "fix-1", "/w/fix", "--paths", "a", "--with="], "--with needs a zone"),
        (&["request", "core", "fix-1", "/w/fix", "--paths", "a", "--with", ""], "--with needs a zone"),
        (&["request", "core", "fix-1", "/w/fix", "--with", "--paths", "a"], "--with needs a zone"),
        (&["request", "core", "fix-1", "/w/fix", "--paths", "a", "--with", "-x"], "--with needs a zone, not \"-x\""),
        (
            &["request", "core", "fix-1", "/w/fix", "--paths", "a", "--with", "web", "--with=web"],
            "partner web is given twice",
        ),
        (
            &["request", "core", "fix-1", "/w/fix", "--paths", "a", "--with", "core"],
            "zone core cannot be its own partner",
        ),
        (&["request", "core", "fix-1", "/w/fix", "--paths", "--with", "web"], "--paths needs at least one path"),
        (
            &["request", "core", "fix-1", "/w/fix", "--paths", "a", "--with", "web", "b"],
            "unexpected word \"b\" in folder request",
        ),
        (
            &["request", "core", "--with", "web", "fix-1", "/w/fix", "--paths", "a"],
            "unknown folder request argument \"--with\"",
        ),
    ];
    for (words, expected) in rows {
        assert_eq!(parse(words), Err(expected.to_string()), "{words:?}");
    }
}

#[test]
fn request_refuses_without_paths_with_none_after_it_or_given_twice() {
    assert_eq!(
        parse(&["request", "core", "fix-1", "/w/fix"]),
        Err("folder request needs --paths <path>...".to_string())
    );
    assert_eq!(
        parse(&["request", "core", "fix-1", "/w/fix", "--paths"]),
        Err("--paths needs at least one path".to_string())
    );
    assert_eq!(
        parse(&["request", "core", "fix-1", "/w/fix", "--paths", "a", "--paths", "b"]),
        Err("--paths is given twice".to_string())
    );
}

#[test]
fn request_refuses_start_a_dashed_path_and_a_missing_name() {
    assert_eq!(
        parse(&["request", "core", "fix-1", "/w/fix", "--paths", "a", "--start", "main"]),
        Err("unknown folder request argument \"--start\"".to_string())
    );
    assert_eq!(
        parse(&["request", "core", "/w/fix", "--paths", "a"]),
        Err("folder request needs <zone> <request> <path>".to_string())
    );
}

#[test]
fn remove_takes_one_path() {
    assert_eq!(parse(&["remove", "/w/fix"]), Ok(FolderCommand::Remove { path: PathBuf::from("/w/fix") }));
    assert_eq!(parse(&["remove"]), Err("folder remove needs <path>".to_string()));
    assert_eq!(parse(&["remove", "/w/a", "/w/b"]), Err("unexpected word \"/w/b\" in folder remove".to_string()));
}

#[test]
fn refresh_takes_one_path() {
    let rows: Vec<(&[&str], Result<FolderCommand, String>)> = vec![
        (&["refresh", "/w/zone"], Ok(FolderCommand::Refresh { path: PathBuf::from("/w/zone") })),
        (&["refresh"], Err("folder refresh needs <path>".to_string())),
        (&["refresh", "/w/a", "/w/b"], Err("unexpected word \"/w/b\" in folder refresh".to_string())),
        (&["refresh", "--start"], Err("unknown folder refresh argument \"--start\"".to_string())),
    ];
    for (words, expected) in rows {
        assert_eq!(parse(words), expected, "{words:?}");
    }
}

#[test]
fn attempts_takes_a_positive_count_and_a_dir() {
    assert_eq!(
        parse(&["attempts", "3", "/w/tries"]),
        Ok(FolderCommand::Attempts { n: 3, dir: PathBuf::from("/w/tries") })
    );
    assert_eq!(
        parse(&["attempts", "4294967295", "/w/tries"]),
        Ok(FolderCommand::Attempts { n: 4_294_967_295, dir: PathBuf::from("/w/tries") })
    );
    assert_eq!(parse(&["attempts", "3"]), Err("folder attempts needs <n> <dir>".to_string()));
}

#[test]
fn attempts_refuses_zero_and_counts_not_written_plainly() {
    for word in ["0", "03", "+3", "three", "4294967296", "3.0"] {
        assert_eq!(
            parse(&["attempts", word, "/w/tries"]),
            Err(format!("<n> is a whole number from 1 to 4294967295, not {word:?}")),
            "{word}"
        );
    }
    assert_eq!(parse(&["attempts", "-3", "/w/tries"]), Err("unknown folder attempts argument \"-3\"".to_string()));
}

#[test]
fn keep_takes_one_positive_count() {
    assert_eq!(parse(&["keep", "2"]), Ok(FolderCommand::Keep { k: 2 }));
    assert_eq!(parse(&["keep", "0"]), Err("<k> is a whole number from 1 to 4294967295, not \"0\"".to_string()));
    assert_eq!(parse(&["keep"]), Err("folder keep needs <k>".to_string()));
    assert_eq!(parse(&["keep", "1", "2"]), Err("unexpected word \"2\" in folder keep".to_string()));
}

#[test]
fn drop_takes_no_words() {
    assert_eq!(parse(&["drop"]), Ok(FolderCommand::Drop));
    assert_eq!(parse(&["drop", "1"]), Err("unexpected word \"1\" in folder drop".to_string()));
}

#[test]
fn an_empty_word_is_refused() {
    assert_eq!(parse(&["remove", ""]), Err("unknown folder remove argument \"\"".to_string()));
}

#[test]
fn no_sub_verb_or_an_unknown_one_is_refused() {
    assert_eq!(
        parse(&[]),
        Err("folder needs a sub-verb: zone, request, remove, refresh, attempts, keep or drop".to_string())
    );
    assert_eq!(parse(&["move", "/w/a"]), Err("unknown folder sub-verb \"move\"".to_string()));
}
