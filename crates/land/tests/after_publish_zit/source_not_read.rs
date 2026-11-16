//! The squash source is not read when the zone holds its record (#6). Zone `land` is squashed at A1 and the squash
//! kept; after_publish(b, others=[land]) replays A1 onto b's origin as A1' and records A1 = A1' (fixture replayed).
//! The kept squash lands as a merge and publishes; after_publish(land), run through a git that logs every argument
//! list (shims tracer), takes A1' through the record without naming A1 to git as an object: the only arguments
//! that hold A1's id are the record's own ref name. The zone is put on origin.

use crate::fixture::{land_and_publish, origin_untouched, record_ref, refs, replayed, zone, Fixture, ACTOR, PUBLISHED};
use crate::shims::{traced_calls, tracer};
use land::{after_publish, ZoneMove};

#[test]
fn a_held_record_moves_the_zone_without_reading_the_source() {
    let fx = Fixture::new();
    let replayed = replayed(&fx);
    let origin = land_and_publish(&fx, &replayed.land_squash);
    let (git, log) = tracer(&fx);
    let moves = after_publish(&git, &refs(), &zone(PUBLISHED, vec![]), &[], ACTOR);
    let expected = vec![ZoneMove::Moved {
        zone: PUBLISHED.to_string(),
        old: replayed.a1_replayed.clone(),
        new: origin.clone(),
        rebase: vec![],
    }];
    assert!(matches!(&moves, Ok(answer) if *answer == expected), "{moves:?}");
    assert_eq!(fx.branch("zone/land"), origin);
    let calls = traced_calls(&log);
    // The log is after_publish's own: it read the zone branch through the tracer.
    assert!(calls.iter().flatten().any(|arg| arg == "refs/heads/zone/land"), "{calls:?}");
    let record = record_ref(PUBLISHED, &replayed.a1);
    let names_source = |arg: &String| arg.contains(replayed.a1.as_str()) && *arg != record;
    let source_reads: Vec<&Vec<String>> = calls.iter().filter(|call| call.iter().any(names_source)).collect();
    assert_eq!(source_reads, Vec::<&Vec<String>>::new());
    origin_untouched(&fx, &origin);
}
