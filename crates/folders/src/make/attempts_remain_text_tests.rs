//! attempts_remain_text: the folder, every attempt left, and the next step for kept, dropped and orphan attempts.

use std::path::Path;

use super::attempts_remain_text::attempts_remain_text;

#[test]
fn names_the_folder_every_attempt_and_the_next_step() {
    let attempts = ["refs/heads/try/z/r/1".to_string(), "/wt/try-2".to_string()];
    assert_eq!(
        attempts_remain_text(Path::new("/wt/r1"), &attempts),
        "/wt/r1 was not removed: attempts of its request remain: refs/heads/try/z/r/1, /wt/try-2; nothing \
         changed; next step: have the harness keep one attempt or drop them all; an attempt folder that lost its \
         branch is re-made from its snapshot when the harness keeps or drops, and one with no snapshot: copy out \
         anything wanted and remove it by hand"
    );
}
