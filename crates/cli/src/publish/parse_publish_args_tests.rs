//! parse_publish_args: no word pushes, --no-push does not; a doubled --no-push or any other word is refused,
//! naming the word.

use super::parse_publish_args;

#[test]
fn publish_words_give_push_or_the_reason() {
    let rows: [(&[&str], Result<bool, &str>); 6] = [
        (&[], Ok(true)),
        (&["--no-push"], Ok(false)),
        (&["--no-push", "--no-push"], Err("--no-push is given twice")),
        (&["now"], Err("unexpected word \"now\"")),
        (&["--no-push", "now"], Err("unexpected word \"now\"")),
        (&["--push"], Err("unexpected word \"--push\"")),
    ];
    for (rest, expected) in rows {
        let rest = rest.iter().map(ToString::to_string).collect::<Vec<_>>();
        assert_eq!(parse_publish_args(&rest), expected.map_err(ToString::to_string), "{rest:?}");
    }
}
