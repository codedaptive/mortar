use super::actor_kinds;
use crate::json::parse_json;

/// Records as oplog writes them (encode_op_record's members, encode_valued_record's values), for two actors.
const LOG: &str = r#"{"verb":"log","outcome":"listed","records":[
  {"op_id":"op-1","key":"k1","kind":"save","actor":"a/r1","reason":"save","at":1,"phase":"intent","moves":[]},
  {"op_id":"op-1","key":"k1","kind":"save","actor":"a/r1","reason":"save","at":2,"phase":"done","moves":[]},
  {"op_id":"op-2","key":"k2","kind":"save","actor":"a/rc","reason":"save","at":3,"phase":"done","moves":[]},
  {"op_id":"op-3","key":"k3","kind":"join-fast-forward","actor":"a/rc","reason":"r","at":4,"phase":"done","moves":[]},
  {"op_id":"op-4","key":"k4","kind":"snapshot","actor":"a/r1","reason":"r","at":5,"phase":"done","moves":[]},
  {"op_id":"op-5","key":"k5","kind":"snapshot","actor":"a/r1","reason":"r","at":6,"phase":"done","moves":[]},
  {"op_id":"op-6","key":"k6","kind":"join-rebase","actor":"a/r1","reason":"r","at":7,"phase":"failed","moves":[]},
  {"op_id":"op-7","key":"k7","kind":"join-fast-forward","actor":"a/r1","reason":"r","at":8,"phase":"done","moves":[]}
]}"#;

#[test]
fn each_done_kind_of_the_actor_is_listed_once_in_log_order() {
    let report = parse_json(LOG).expect("the test's JSON parses");
    assert_eq!(actor_kinds(&report, "a/r1"), vec!["save", "snapshot", "join-fast-forward"]);
    assert_eq!(actor_kinds(&report, "a/rc"), vec!["save", "join-fast-forward"]);
}

#[test]
fn a_join_rebase_recording_the_join_state_reads_as_resolve() {
    let text = r#"{"records":[
      {"op_id":"op-1","kind":"join-rebase","actor":"a/r1","phase":"done","moves":[],"values":[{"name":"folder_tree","value":"abc"},{"name":"join_state","value":"{}"}]},
      {"op_id":"op-2","kind":"join-rebase","actor":"a/r1","phase":"done","moves":[],"values":[{"name":"folder_tree","value":"abc"}]},
      {"op_id":"op-3","kind":"join-rebase","actor":"a/r1","phase":"done","moves":[]}
    ]}"#;
    let report = parse_json(text).expect("the test's JSON parses");
    assert_eq!(actor_kinds(&report, "a/r1"), vec!["resolve (join-rebase)", "join-rebase"]);
}

#[test]
fn an_actor_with_no_records_or_a_report_without_records_gives_nothing() {
    let report = parse_json(LOG).expect("the test's JSON parses");
    assert_eq!(actor_kinds(&report, "b/r2"), Vec::<String>::new());
    assert_eq!(actor_kinds(&parse_json(r#"{"outcome":"empty"}"#).expect("parses"), "a/r1"), Vec::<String>::new());
}
