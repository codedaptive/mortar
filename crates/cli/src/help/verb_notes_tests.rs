//! verb_notes: squash's two notes name --message's place and its missing trailers, then the two shapes read back
//! wrong; publish's note names --no-push and pushed; commit-files's two notes name its branch and lock in each
//! folder and the folders that refuse it, then --message; folder's, would-merge's and commit-files's notes are
//! pinned in full by verb_help_lines_tests; every other verb has none.
//! Unit test: verb_notes and the verb table are pure.

use super::verb_notes;
use crate::args::VERBS;

#[test]
fn squash_notes_name_the_message_its_trailers_and_the_two_misread_shapes() {
    let notes = verb_notes("squash");
    assert_eq!(notes.len(), 2, "{notes:?}");
    for token in ["--message <text>", "no trailers"] {
        assert!(notes[0].contains(token), "{token}: {}", notes[0]);
    }
    for token in ["\"- x: y\"", "\"<x>: <n> requests\""] {
        assert!(notes[1].contains(token), "{token}: {}", notes[1]);
    }
}

#[test]
fn publish_note_names_no_push_and_when_pushed_is_true() {
    let notes = verb_notes("publish");
    assert_eq!(notes.len(), 1, "{notes:?}");
    for token in ["--no-push", "pushed is true only when an earlier publish"] {
        assert!(notes[0].contains(token), "{token}: {}", notes[0]);
    }
}

#[test]
fn commit_files_notes_name_each_folders_branch_and_lock_then_the_message() {
    let notes = verb_notes("commit-files");
    assert_eq!(notes.len(), 2, "{notes:?}");
    for token in ["zone/<zone>", "join-<zone>", "integration", "under land", "request or attempt folder", "mr save"] {
        assert!(notes[0].contains(token), "{token}: {}", notes[0]);
    }
    assert!(notes[1].starts_with("--message <text>"), "{}", notes[1]);
}

#[test]
fn every_other_verb_has_no_notes() {
    let noted = ["folder", "squash", "publish", "would-merge", "commit-files"];
    for verb in VERBS.iter().map(|verb| verb.name).chain(["--version"]).filter(|name| !noted.contains(name)) {
        assert!(verb_notes(verb).is_empty(), "{verb}");
    }
}
