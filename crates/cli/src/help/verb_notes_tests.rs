//! verb_notes: squash's two notes name --message's place and its missing trailers, then the two shapes read back
//! wrong; publish's note names --no-push and pushed; every other verb has none.
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
fn every_other_verb_has_no_notes() {
    let noted = ["squash", "publish"];
    for verb in VERBS.iter().map(|verb| verb.name).chain(["--version"]).filter(|name| !noted.contains(name)) {
        assert!(verb_notes(verb).is_empty(), "{verb}");
    }
}
