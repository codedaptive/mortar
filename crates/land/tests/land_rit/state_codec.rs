//! Thirteen owed Land -> Oplog JSON/Gitio ID codec cases, through PUBLIC land/back_out/publish and real files.
//! S-109/#1589 narrows the two writer fixtures to reachable quote/backslash zones (unicode also retained).
//! Newline/control-character zones cannot reach today's public writer and are NOT owed; no codec API is exposed.
//! Decoder refusals still use hand-written malformed files. No private production source is copied or included.
use crate::{owed_ops::{locked, raw, refs}, support::Fixture};
use gitio::{Oid, RunOpts};
use land::{BackOutAnswer, LandAnswer, LandError, LandingStateError, PublishAnswer, Squash};
use oplog::ChangeId;
use std::{fs, path::PathBuf};

const CID: &str = "klmnopqrstuvwxyzklmnopqrstuvwxyz";
const INTEGRATION: &str = "refs/heads/integration";
struct State { f: Fixture, squash: Squash, origin: Oid, integration: Oid, path: PathBuf }
impl State {
    fn new(zone: &str) -> Self {
        let f = Fixture::new(); let base = f.git.read_ref("refs/heads/main").unwrap().unwrap();
        let base_tree = f.git.commit_info(&base).unwrap().tree;
        let source_tree = f.git.commit_info(&f.source).unwrap().tree;
        let origin = f.git.commit_tree(&base_tree, std::slice::from_ref(&base), "independent origin\n", None, None).unwrap();
        let message = format!("fixture squash\n\nMortar-Change-Id: {CID}\nMortar-Zone: {zone}\n");
        let commit = f.git.commit_tree(&source_tree, &[base], &message, None, None).unwrap();
        let squash = Squash { commit, change_id: ChangeId::parse(CID).unwrap(), source: f.source.clone(), requests: vec![] };
        // Independent Git object oracle: origin adds no file change, so merge tree is the source tree.
        let integration = f.git.commit_tree(&source_tree, &[origin.clone(), squash.commit.clone()],
            &format!("land {CID} ({zone})\n"), None, None).unwrap();
        assert_ne!(squash.commit, origin); assert_ne!(integration, origin); assert_ne!(integration, squash.commit);
        raw(&f, &["update-ref", "refs/heads/main", origin.as_str()]); raw(&f, &["update-ref", INTEGRATION, origin.as_str()]);
        let path = f.common.join("mortar/landing.json"); Self { f, squash, origin, integration, path }
    }
    // Literal JSON layout + controlled input values; deliberately no call to either kit's string encoder.
    fn text(&self, literal_json_zone: &str) -> String {
        format!("{{\"change_id\":\"klmnopqrstuvwxyzklmnopqrstuvwxyz\",\"zone\":\"{literal_json_zone}\",\"squash\":\"{}\",\"origin_at_start\":\"{}\",\"integration\":\"{}\"}}\n",
            self.squash.commit, self.origin, self.integration)
    }
    fn seed(&self, text: &str) {
        fs::create_dir_all(self.path.parent().unwrap()).unwrap(); fs::write(&self.path, text).unwrap();
    }
    fn observe(&self) -> (Vec<u8>, Option<Vec<u8>>) {
        (self.f.git.run(&["for-each-ref", "--format=%(refname) %(objectname)"], &RunOpts::default()).unwrap().stdout,
            fs::read(oplog::oplog_path(&self.f.common)).ok())
    }
    fn write_via_land(&self) {
        match locked(&self.f, || land::land(&self.f.git, &refs(), &self.squash, "state-rit")) {
            LandAnswer::Landed { integration } => assert_eq!(integration, self.integration),
            other => panic!("expected real merge landing: {other:?}"),
        }
        assert_eq!(self.f.git.read_ref(INTEGRATION).unwrap(), Some(self.integration.clone()));
    }
    fn seed_for_read(&self, text: &str) {
        raw(&self.f, &["update-ref", INTEGRATION, self.integration.as_str()]); self.seed(text);
    }
    fn assert_public_read(&self, text: &str, zone_ref: Option<&str>) {
        assert_eq!(fs::read(&self.path).unwrap(), text.as_bytes()); let before = self.observe();
        match locked(&self.f, || land::land(&self.f.git, &refs(), &self.squash, "state-rit")) {
            LandAnswer::Landed { integration } => assert_eq!(integration, self.integration),
            other => panic!("decoded squash/integration must select repeat: {other:?}"),
        }
        let another = Squash { commit: self.f.source.clone(), ..self.squash.clone() };
        assert_ne!(another.commit, self.squash.commit);
        match locked(&self.f, || land::land(&self.f.git, &refs(), &another, "state-rit")) {
            LandAnswer::Busy { change_id } => assert_eq!(change_id.as_str(), CID),
            other => panic!("decoded change ID must be reported busy: {other:?}"),
        }
        assert_eq!(self.observe(), before); assert_eq!(fs::read(&self.path).unwrap(), text.as_bytes());
        match locked(&self.f, || land::back_out(&self.f.git, &refs(), "state-rit")) {
            BackOutAnswer::BackedOut { change_id } => assert_eq!(change_id.as_str(), CID),
            other => panic!("expected decoded origin and change ID in back-out: {other:?}"),
        }
        assert_eq!(self.f.git.read_ref(INTEGRATION).unwrap(), Some(self.origin.clone())); assert!(!self.path.exists());
        if let Some(zone_ref) = zone_ref {
            // A second real read observes decoded zone/squash via the cleanup ref. Local origin already new:
            // publish does not contact a remote or exercise the excluded S-103 interrupted-push behavior.
            self.seed(text); raw(&self.f, &["update-ref", "refs/heads/main", self.integration.as_str()]);
            raw(&self.f, &["update-ref", INTEGRATION, self.integration.as_str()]);
            raw(&self.f, &["update-ref", zone_ref, self.squash.commit.as_str()]);
            raw(&self.f, &["update-ref", "refs/mortar/squash/decoy", self.squash.commit.as_str()]);
            assert_eq!(locked(&self.f, || land::publish(&self.f.git, &refs(), "state-rit")),
                PublishAnswer::Published { origin: self.integration.clone() });
            assert_eq!(self.f.git.read_ref(zone_ref).unwrap(), None);
            assert_eq!(self.f.git.read_ref("refs/mortar/squash/decoy").unwrap(), Some(self.squash.commit.clone()));
            assert!(!self.path.exists());
            assert!(oplog::read_op_log(&self.f.common).unwrap().records.iter().all(|r| r.kind != "publish-push"));
        }
    }
    fn refusal(&self, text: &str) -> String {
        self.seed(text); let before = self.observe();
        let outcome = locked(&self.f, || Ok::<_, LandError>(land::land(&self.f.git, &refs(), &self.squash, "state-rit")));
        assert_eq!(self.observe(), before); assert_eq!(fs::read(&self.path).unwrap(), text.as_bytes());
        assert!(!self.path.with_file_name("landing.json.tmp").exists());
        match outcome {
            Err(LandError::State(LandingStateError::Corrupt { path, reason })) => { assert_eq!(path, self.path); reason }
            other => panic!("expected public Corrupt for {text:?}: {other:?}"),
        }
    }
}

#[test]
fn rit_a_state_is_one_object_in_key_order_with_a_trailing_newline() {
    let s = State::new("land"); s.write_via_land(); let expected = s.text("land");
    assert_eq!(fs::read(&s.path).unwrap(), expected.as_bytes());
    s.assert_public_read(&expected, Some("refs/mortar/squash/land"));
}
#[test]
fn rit_a_zone_needing_reachable_escapes_is_escaped_and_read_back() {
    let s = State::new("a\"b\\c"); s.write_via_land(); let expected = s.text(r#"a\"b\\c"#);
    assert_eq!(fs::read(&s.path).unwrap(), expected.as_bytes()); s.assert_public_read(&expected, None);
}
#[test]
fn rit_hand_written_text_reads_into_every_publicly_observable_field() {
    let s = State::new("land"); let text = s.text("land"); s.seed_for_read(&text);
    s.assert_public_read(&text, Some("refs/mortar/squash/land"));
}
#[test]
fn rit_the_trailing_newline_is_optional() {
    let s = State::new("land"); let text = s.text("land"); let text = text.strip_suffix('\n').unwrap();
    s.seed_for_read(text); s.assert_public_read(text, Some("refs/mortar/squash/land"));
}
#[test]
fn rit_escapes_encode_never_writes_are_read_as_json() {
    let s = State::new("land/x"); let text = s.text(r#"land\/x"#); s.seed_for_read(&text);
    s.assert_public_read(&text, Some("refs/mortar/squash/land/x"));
}
#[test]
fn rit_a_missing_landing_state_key_is_refused() {
    let s = State::new("land"); let text = s.text("land").replace(",\"zone\":\"land\"", "");
    assert_eq!(s.refusal(&text), "expected ,\"zone\": at byte 47");
}
#[test]
fn rit_landing_state_keys_out_of_order_are_refused() {
    let s = State::new("land");
    let old = format!("\"origin_at_start\":\"{}\",\"integration\":\"{}\"", s.origin, s.integration);
    let new = format!("\"integration\":\"{}\",\"origin_at_start\":\"{}\"", s.integration, s.origin);
    let text = s.text("land").replace(&old, &new);
    assert!(text.find("\"integration\"") < text.find("\"origin_at_start\""));
    assert_eq!(s.refusal(&text), "expected ,\"origin_at_start\": at byte 113");
}
#[test]
fn rit_a_bad_object_id_is_refused_naming_the_byte() {
    let s = State::new("land"); let text = s.text("land").replace(s.squash.commit.as_str(), "1111");
    let reason = s.refusal(&text); assert!(reason.starts_with("at byte 71: "), "{reason}");
}
#[test]
fn rit_a_bad_change_id_is_refused_naming_the_byte() {
    let s = State::new("land"); let text = s.text("land").replace(CID, "abcdefabcdefabcdefabcdefabcdefab");
    let reason = s.refusal(&text); assert!(reason.starts_with("at byte 13: "), "{reason}");
}
#[test]
fn rit_text_after_the_landing_state_brace_is_refused() {
    let s = State::new("land");
    for junk in [" ", "\n", "x", "{}"] {
        assert_eq!(s.refusal(&format!("{}{junk}", s.text("land"))),
            "expected the closing brace and the end of the text at byte 231", "{junk:?}");
    }
}
#[test]
fn rit_landing_state_whitespace_and_empty_text_are_refused() {
    let s = State::new("land"); let text = s.text("land").replace("\"zone\":", "\"zone\": ");
    assert_eq!(s.refusal(&text), "expected \" at byte 55");
    assert_eq!(s.refusal(""), "expected {\"change_id\": at byte 0");
}
#[test]
fn rit_a_landing_state_string_that_ends_early_is_refused_saying_why() {
    let s = State::new("land");
    assert_eq!(s.refusal("{\"change_id\":\"klmn"),
        "the landing state's text could not be read: it ends inside the string starting at byte 13");
}
#[test]
fn rit_every_public_writer_state_round_trips_with_reachable_zone_inputs() {
    for (zone, json_zone, zone_ref) in [("land", "land", Some("refs/mortar/squash/land")),
        ("a\"b\\c", r#"a\"b\\c"#, None), ("zone \"q\" \\ é", r#"zone \"q\" \\ é"#, None)] {
        let s = State::new(zone); s.write_via_land(); let expected = s.text(json_zone);
        assert_eq!(fs::read(&s.path).unwrap(), expected.as_bytes()); s.assert_public_read(&expected, zone_ref);
    }
}
