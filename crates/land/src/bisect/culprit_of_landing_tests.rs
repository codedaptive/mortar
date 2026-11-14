//! culprit_of_landing: a fast-forward reads the commit's own trailers, a merge its second parent's, the
//! Culprit always names the landing commit, a squash without trailers gives None, a commit with 0 or 3
//! parents is refused without reading trailers, and a trailer error is Trailers.

use std::collections::HashMap;

use gitio::Oid;
use oplog::{ChangeId, ChangeIdError, MortarTrailerError, MortarTrailers};

use super::{culprit_of_landing, Culprit, CulpritError};

const ID_COMMIT: &str = "kzlymxnwovpuqtrssrtqupvowxnymlzk";
const ID_FIRST: &str = "zzzzzzzzzzzzzzzzkkkkkkkkkkkkkkkk";
const ID_SECOND: &str = "mmmmnnnnoooopppqqqqrrrrssssttttu";

fn oid(hex_digit: char) -> Oid {
    Oid::parse(&hex_digit.to_string().repeat(40)).unwrap()
}

fn trailers(id: &str, zone: &str) -> MortarTrailers {
    MortarTrailers { change_id: Some(ChangeId::parse(id).unwrap()), request: None, zone: Some(zone.to_string()) }
}

/// The landing commit 'c', its first parent '1' and second parent '2', each with its own trailers.
fn table() -> HashMap<Oid, MortarTrailers> {
    HashMap::from([
        (oid('c'), trailers(ID_COMMIT, "commit-zone")),
        (oid('1'), trailers(ID_FIRST, "first-zone")),
        (oid('2'), trailers(ID_SECOND, "second-zone")),
    ])
}

/// Runs culprit_of_landing against `table`, returning its result and every oid trailers_of was asked for.
fn run(
    commit: &Oid,
    parents: &[Oid],
    table: &HashMap<Oid, MortarTrailers>,
) -> (Result<Culprit, CulpritError>, Vec<Oid>) {
    let mut asked = Vec::new();
    let result = culprit_of_landing(commit, parents, |o: &Oid| {
        asked.push(o.clone());
        Ok(table[o].clone())
    });
    (result, asked)
}

#[test]
fn fast_forward_reads_the_commits_own_trailers() {
    let (result, asked) = run(&oid('c'), &[oid('1')], &table());
    let expected = Culprit {
        commit: oid('c'),
        change_id: Some(ChangeId::parse(ID_COMMIT).unwrap()),
        zone: Some("commit-zone".to_string()),
    };
    assert_eq!(result.unwrap(), expected);
    assert_eq!(asked, vec![oid('c')]);
}

#[test]
fn merge_reads_the_second_parents_trailers_and_names_the_merge() {
    let (result, asked) = run(&oid('c'), &[oid('1'), oid('2')], &table());
    let expected = Culprit {
        commit: oid('c'),
        change_id: Some(ChangeId::parse(ID_SECOND).unwrap()),
        zone: Some("second-zone".to_string()),
    };
    assert_eq!(result.unwrap(), expected);
    assert_eq!(asked, vec![oid('2')]);
}

#[test]
fn a_squash_without_mortar_trailers_is_still_named() {
    let bare = HashMap::from([(oid('c'), MortarTrailers::default())]);
    let (result, _) = run(&oid('c'), &[oid('1')], &bare);
    assert_eq!(result.unwrap(), Culprit { commit: oid('c'), change_id: None, zone: None });
}

#[test]
fn no_parents_is_not_a_landing_and_reads_no_trailers() {
    let (result, asked) = run(&oid('c'), &[], &table());
    assert!(
        matches!(result, Err(CulpritError::NotALanding { ref commit, parents: 0 }) if *commit == oid('c')),
        "{result:?}"
    );
    assert!(asked.is_empty(), "trailers_of was asked for {asked:?}");
}

#[test]
fn three_parents_is_not_a_landing_and_reads_no_trailers() {
    let (result, asked) = run(&oid('c'), &[oid('1'), oid('2'), oid('3')], &table());
    assert!(
        matches!(result, Err(CulpritError::NotALanding { ref commit, parents: 3 }) if *commit == oid('c')),
        "{result:?}"
    );
    assert!(asked.is_empty(), "trailers_of was asked for {asked:?}");
}

#[test]
fn a_trailer_error_is_trailers() {
    let result = culprit_of_landing(&oid('c'), &[oid('1'), oid('2')], |_: &Oid| {
        Err(MortarTrailerError::ChangeId(ChangeIdError::Malformed {
            text: "abc".to_string(),
            reason: "it has 3 chars".to_string(),
        }))
    });
    assert!(
        matches!(
            result,
            Err(CulpritError::Trailers(MortarTrailerError::ChangeId(ChangeIdError::Malformed { ref text, .. })))
                if text == "abc"
        ),
        "{result:?}"
    );
}
