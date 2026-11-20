//! squash_from_parts: all parts present gives the exact Squash, zero requests an empty list; a missing change ID
//! or squash-of is TrailerMissing naming it; a malformed change ID or squash-of is TrailerMalformed with its text;
//! a Random change ID error passes through as ChangeId; request lines that miss the subject's count are
//! RequestsMalformed. MortarTrailers, ChangeId, Oid and ChangeIdError are built as data; refusals are
//! checked by their fields, never their Display (it embeds gitio's Oid Display).

use super::{squash_from_parts, Squash, SquashError};
use gitio::Oid;
use oplog::{ChangeId, ChangeIdError, MortarTrailers};

const COMMIT: &str = "3333333333333333333333333333333333333333";
const SOURCE: &str = "4444444444444444444444444444444444444444";
const CHANGE: &str = "klmnopqrstuvwxyzklmnopqrstuvwxyz";
const MESSAGE: &str =
    "land: 2 requests\n\n- r2: second\n- r1: first\n\nMortar-Change-Id: klmnopqrstuvwxyzklmnopqrstuvwxyz\n";

/// The one helper building values only a constructor makes (Oid::parse, ChangeId::parse).
fn oid(hex: &str) -> Oid {
    Oid::parse(hex).unwrap()
}

fn change_id() -> ChangeId {
    ChangeId::parse(CHANGE).unwrap()
}

fn picked() -> Result<MortarTrailers, ChangeIdError> {
    Ok(MortarTrailers { change_id: Some(change_id()), request: None, zone: Some("land".to_string()) })
}

fn pairs(squash_of: &str) -> Vec<(String, String)> {
    vec![
        ("Mortar-Change-Id".to_string(), CHANGE.to_string()),
        ("Mortar-Zone".to_string(), "land".to_string()),
        ("Mortar-Squash-Of".to_string(), squash_of.to_string()),
    ]
}

#[test]
fn all_parts_present_give_the_exact_squash() {
    let squash = squash_from_parts(oid(COMMIT), picked(), &pairs(SOURCE), MESSAGE).unwrap();
    let expected = Squash {
        commit: oid(COMMIT),
        change_id: change_id(),
        source: oid(SOURCE),
        requests: vec!["r2".to_string(), "r1".to_string()],
    };
    assert_eq!(squash, expected);
}

#[test]
fn zero_requests_give_an_empty_list() {
    let message = "land: 0 requests\n\nMortar-Change-Id: klmnopqrstuvwxyzklmnopqrstuvwxyz\n";
    let squash = squash_from_parts(oid(COMMIT), picked(), &pairs(SOURCE), message).unwrap();
    assert_eq!(squash.requests, Vec::<String>::new());
}

#[test]
fn a_missing_change_id_is_trailer_missing() {
    let picked = Ok(MortarTrailers { change_id: None, request: None, zone: Some("land".to_string()) });
    let error = squash_from_parts(oid(COMMIT), picked, &pairs(SOURCE), MESSAGE).unwrap_err();
    assert!(
        matches!(&error, SquashError::TrailerMissing { commit, trailer }
            if *commit == oid(COMMIT) && trailer == "Mortar-Change-Id"),
        "{error:?}"
    );
}

#[test]
fn a_malformed_change_id_is_trailer_malformed_with_its_text() {
    let picked = Err(ChangeIdError::Malformed { text: "abc".to_string(), reason: "r".to_string() });
    let error = squash_from_parts(oid(COMMIT), picked, &pairs(SOURCE), MESSAGE).unwrap_err();
    assert!(
        matches!(&error, SquashError::TrailerMalformed { commit, trailer, value }
            if *commit == oid(COMMIT) && trailer == "Mortar-Change-Id" && value == "abc"),
        "{error:?}"
    );
}

#[test]
fn a_random_change_id_error_passes_through_as_change_id() {
    let picked = Err(ChangeIdError::Random { source: std::io::Error::other("no urandom") });
    let error = squash_from_parts(oid(COMMIT), picked, &pairs(SOURCE), MESSAGE).unwrap_err();
    assert!(matches!(&error, SquashError::ChangeId(ChangeIdError::Random { .. })), "{error:?}");
}

#[test]
fn a_missing_squash_of_is_trailer_missing() {
    let trailers = vec![("Mortar-Change-Id".to_string(), CHANGE.to_string())];
    let error = squash_from_parts(oid(COMMIT), picked(), &trailers, MESSAGE).unwrap_err();
    assert!(
        matches!(&error, SquashError::TrailerMissing { commit, trailer }
            if *commit == oid(COMMIT) && trailer == "Mortar-Squash-Of"),
        "{error:?}"
    );
}

#[test]
fn a_malformed_squash_of_is_trailer_malformed_with_its_text() {
    let error = squash_from_parts(oid(COMMIT), picked(), &pairs("not-an-oid"), MESSAGE).unwrap_err();
    assert!(
        matches!(&error, SquashError::TrailerMalformed { commit, trailer, value }
            if *commit == oid(COMMIT) && trailer == "Mortar-Squash-Of" && value == "not-an-oid"),
        "{error:?}"
    );
}

#[test]
fn request_lines_that_miss_the_count_are_requests_malformed() {
    let message = "land: 3 requests\n\n- r2: second\n- r1: first\n";
    let error = squash_from_parts(oid(COMMIT), picked(), &pairs(SOURCE), message).unwrap_err();
    assert!(matches!(&error, SquashError::RequestsMalformed { commit } if *commit == oid(COMMIT)), "{error:?}");
}
