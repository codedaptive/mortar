//! hooked_message_reads_back: kept trailers and request lines read back, extra trailers after Mortar's are allowed;
//! a missing or changed Mortar trailer, or a later value of one, names that trailer; changed request lines name
//! "requests".

use super::hooked_message_reads_back;
use crate::squash::HookedSquashError;

const ID: &str = "c1a2b3c4";
const SOURCE: &str = "4444444444444444444444444444444444444444";

/// A row: its name, the hooked message's trailers, the hooked text, and the part Unreadable names (None: Ok).
type Row = (&'static str, Vec<(String, String)>, &'static str, Option<&'static str>);

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    list.iter().map(|(key, value)| (key.to_string(), value.to_string())).collect()
}

#[test]
fn a_hooked_message_reads_back_only_with_its_trailers_and_requests() {
    let written = pairs(&[("Mortar-Change-Id", ID), ("Mortar-Zone", "land"), ("Mortar-Squash-Of", SOURCE)]);
    let requests = vec!["r1".to_string()];
    let kept = "Fix\n\n- r1: add z\n\nMortar-Change-Id: c1a2b3c4\nMortar-Zone: land\n\
                Mortar-Squash-Of: 4444444444444444444444444444444444444444\n";
    let rows: [Row; 7] = [
        ("all kept", written.clone(), kept, None),
        (
            "missing change id",
            pairs(&[("Mortar-Zone", "land"), ("Mortar-Squash-Of", SOURCE)]),
            "Fix\n\n- r1: add z\n\nMortar-Zone: land\nMortar-Squash-Of: 4444444444444444444444444444444444444444\n",
            Some("Mortar-Change-Id"),
        ),
        (
            "changed zone",
            pairs(&[("Mortar-Change-Id", ID), ("Mortar-Zone", "join"), ("Mortar-Squash-Of", SOURCE)]),
            "Fix\n\n- r1: add z\n\nMortar-Change-Id: c1a2b3c4\nMortar-Zone: join\n\
             Mortar-Squash-Of: 4444444444444444444444444444444444444444\n",
            Some("Mortar-Zone"),
        ),
        (
            "missing squash of",
            pairs(&[("Mortar-Change-Id", ID), ("Mortar-Zone", "land")]),
            "Fix\n\n- r1: add z\n\nMortar-Change-Id: c1a2b3c4\nMortar-Zone: land\n",
            Some("Mortar-Squash-Of"),
        ),
        (
            "extra trailers after Mortar's",
            pairs(&[
                ("Mortar-Change-Id", ID),
                ("Mortar-Zone", "land"),
                ("Mortar-Squash-Of", SOURCE),
                ("Signed-off-by", "a"),
            ]),
            "Fix\n\n- r1: add z\n\nMortar-Change-Id: c1a2b3c4\nMortar-Zone: land\n\
             Mortar-Squash-Of: 4444444444444444444444444444444444444444\nSigned-off-by: a\n",
            None,
        ),
        (
            "request lines changed",
            written.clone(),
            "Fix\n\n- r2: add z\n\nMortar-Change-Id: c1a2b3c4\nMortar-Zone: land\n\
             Mortar-Squash-Of: 4444444444444444444444444444444444444444\n",
            Some("requests"),
        ),
        (
            "later duplicate key with another value",
            pairs(&[
                ("Mortar-Change-Id", ID),
                ("Mortar-Zone", "land"),
                ("Mortar-Squash-Of", SOURCE),
                ("Mortar-Zone", "join"),
            ]),
            "Fix\n\n- r1: add z\n\nMortar-Change-Id: c1a2b3c4\nMortar-Zone: land\n\
             Mortar-Squash-Of: 4444444444444444444444444444444444444444\nMortar-Zone: join\n",
            Some("Mortar-Zone"),
        ),
    ];
    let mut failures = Vec::new();
    for (name, hooked_trailers, hooked, expected) in &rows {
        let got = match hooked_message_reads_back(&written, hooked_trailers, &requests, hooked) {
            Ok(()) => None,
            Err(HookedSquashError::Unreadable { part, message }) => Some((part, message)),
            Err(other) => panic!("{name}: expected Unreadable or Ok, got {other:?}"),
        };
        let want = expected.map(|part| (part.to_string(), hooked.to_string()));
        if got != want {
            failures.push(format!("{name}: got {got:?}, want {want:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
