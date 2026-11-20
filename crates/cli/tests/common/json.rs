//! Strict parser of the report's documented JSON schema, including nested objects; no new JSON dependency in the product.
use std::{collections::BTreeMap, process::Output};

#[derive(Debug, PartialEq, Eq)]
pub enum Value { Text(String), Number(u64), Bool(bool), Null, Texts(Vec<String>), Object(Vec<(String, Value)>) }
#[derive(Debug)]
pub struct Report(BTreeMap<String, Value>);
impl Report {
    pub fn text(&self, key: &str) -> &str { match &self.0[key] { Value::Text(s) => s, other => panic!("{key}: {other:?}") } }
    pub fn number(&self, key: &str) -> u64 { match self.0[key] { Value::Number(n) => n, ref other => panic!("{key}: {other:?}") } }
    pub fn flag(&self, key: &str) -> bool { match self.0[key] { Value::Bool(b) => b, ref other => panic!("{key}: {other:?}") } }
    pub fn is_null(&self, key: &str) -> bool { self.0[key] == Value::Null }
    pub fn strings(&self, key: &str) -> &[String] { match &self.0[key] { Value::Texts(v) => v, other => panic!("{key}: {other:?}") } }
    pub fn object(&self, key: &str) -> &[(String, Value)] { match &self.0[key] { Value::Object(v) => v, other => panic!("{key}: {other:?}") } }
    pub fn lines(&self) -> String { self.strings("lines").join("\n") }
}
pub fn check(output: Output, verb: &str, exit: i32, outcome: &str) -> Report {
    let report = read(output, verb, exit);
    assert_eq!(report.text("outcome"), outcome);
    report
}
/// The same strict envelope parser when a case asserts status fields rather than a fixed status outcome.
pub fn read(output: Output, verb: &str, exit: i32) -> Report {
    assert_eq!(output.status.code(), Some(exit), "{output:?}");
    assert!(output.stderr.is_empty(), "unexpected stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let text = stdout.strip_suffix('\n').expect("one JSON line must end with a newline");
    assert!(!text.contains('\n'), "extra stdout or a raw newline in JSON: {stdout}");
    let mut at = 0; take(text, &mut at, b'{');
    let mut fields = BTreeMap::new();
    loop {
        let key = string(text, &mut at); take(text, &mut at, b':');
        let value = value(text, &mut at);
        assert!(fields.insert(key, value).is_none(), "duplicate JSON key");
        match text.as_bytes().get(at) { Some(b',') => at += 1, Some(b'}') => { at += 1; break; }, other => panic!("bad JSON separator {other:?}: {text}") }
    }
    assert_eq!(at, text.len(), "trailing stdout after the JSON object");
    let report = Report(fields);
    assert_eq!(report.text("verb"), verb); assert_eq!(report.number("exit"), exit as u64);
    assert_eq!(report.flag("ok"), exit == 0);
    report
}
fn string(text: &str, at: &mut usize) -> String {
    take(text, at, b'"');
    let (s, end) = oplog::parse_json_string(text, *at).unwrap(); *at = end; s
}
fn take(text: &str, at: &mut usize, expected: u8) {
    assert_eq!(text.as_bytes().get(*at), Some(&expected), "invalid JSON at byte {at}: {text}"); *at += 1;
}
fn value(text: &str, at: &mut usize) -> Value {
    match text.as_bytes().get(*at) {
        Some(b'"') => Value::Text(string(text, at)),
        Some(b'{') => {
            *at += 1; let mut out = Vec::new();
            if text.as_bytes().get(*at) != Some(&b'}') {
                loop {
                    let key = string(text, at); take(text, at, b':');
                    let item = value(text, at);
                    assert!(!out.iter().any(|(k, _)| k == &key), "duplicate JSON key");
                    out.push((key, item));
                    if text.as_bytes().get(*at) == Some(&b',') { *at += 1; } else { break; }
                }
            }
            take(text, at, b'}'); Value::Object(out)
        }
        Some(b'[') => {
            *at += 1; let mut out = Vec::new();
            if text.as_bytes().get(*at) != Some(&b']') {
                loop { out.push(string(text, at)); if text.as_bytes().get(*at) == Some(&b',') { *at += 1; } else { break; } }
            }
            take(text, at, b']'); Value::Texts(out)
        }
        Some(b't') => { assert!(text[*at..].starts_with("true")); *at += 4; Value::Bool(true) }
        Some(b'f') => { assert!(text[*at..].starts_with("false")); *at += 5; Value::Bool(false) }
        Some(b'n') => { assert!(text[*at..].starts_with("null")); *at += 4; Value::Null }
        Some(b'0'..=b'9') => {
            let start = *at;
            while text.as_bytes().get(*at).is_some_and(u8::is_ascii_digit) { *at += 1; }
            let digits = &text[start..*at]; assert!(digits == "0" || !digits.starts_with('0'));
            Value::Number(digits.parse().unwrap())
        }
        other => panic!("unexpected JSON value {other:?}: {text}"),
    }
}
