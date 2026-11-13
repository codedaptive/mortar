//! decode_op_line: every cut-short prefix of a line is Torn; a complete but wrong line is Malformed.

use super::{decode_op_line, LineError, OpPhase};

const LINE: &str = "{\"op_id\":\"op-\\\"7\\\"\",\"key\":\"k\\u0001\",\"kind\":\"land\",\"actor\":\"é主\",\
                    \"reason\":\"\\ud83d\\ude00 \\n\",\"at\":1763030400,\"phase\":\"done\",\
                    \"moves\":[{\"ref\":\"refs/heads/x\",\"old\":null,\"new\":\"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"}],\
                    \"result\":{\"outcome\":\"moved\"}}";

#[test]
fn the_whole_line_decodes() {
    let r = decode_op_line(LINE).unwrap();
    assert_eq!(r.op_id, "op-\"7\"");
    assert_eq!(r.key, "k\u{1}");
    assert_eq!(r.actor, "é主");
    assert_eq!(r.reason, "😀 \n");
    assert_eq!(r.phase, OpPhase::Done);
}

#[test]
fn every_proper_prefix_is_torn() {
    let mut count = 0;
    for (cut, _) in LINE.char_indices() {
        assert_eq!(decode_op_line(&LINE[..cut]), Err(LineError::Torn), "{:?}", &LINE[..cut]);
        count += 1;
    }
    assert_eq!(count, LINE.chars().count());
}

#[test]
fn complete_json_that_is_not_a_record_is_malformed() {
    assert_eq!(
        decode_op_line("{}"),
        Err(LineError::Malformed("missing key \"op_id\"".to_string()))
    );
    assert_eq!(
        decode_op_line("null"),
        Err(LineError::Malformed("the record is not a JSON object".to_string()))
    );
}

#[test]
fn bad_json_is_malformed_not_torn() {
    for line in ["{\"op_id\" \"x\"}", "{\"a\":true}", "x", "{\"a\":\"\\q\"}"] {
        assert!(matches!(decode_op_line(line), Err(LineError::Malformed(_))), "{line}");
    }
}
