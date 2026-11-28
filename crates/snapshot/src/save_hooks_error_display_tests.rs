use crate::{SaveHooksError, SnapshotError};
use gitio::HookRefusal;

fn refused(hook: &'static str, code: Option<i32>) -> SaveHooksError {
    let (stdout, stderr) = (b"lint: out\n".to_vec(), b"lint: err\n".to_vec());
    SaveHooksError::HookRefused(HookRefusal { hook, code, stdout, stderr })
}

#[test]
fn save_hooks_error_shows_one_sentence_per_variant() {
    let rows = [
        (
            refused("pre-commit", Some(1)),
            "the pre-commit hook refused the commit with exit code 1; no commit was written and no branch moved",
        ),
        (
            refused("commit-msg", None),
            "the commit-msg hook was killed by a signal; no commit was written and no branch moved",
        ),
        (
            SaveHooksError::Snapshot(SnapshotError::BadSentence { reason: "it is blank".into() }),
            "the save sentence is refused: it is blank",
        ),
    ];
    for (error, expected) in rows {
        assert_eq!(error.to_string(), expected, "variant {error:?}");
    }
}
