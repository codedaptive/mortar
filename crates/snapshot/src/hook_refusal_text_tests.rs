use crate::hook_refusal_text::hook_refusal_text;
use gitio::HookRefusal;

#[test]
fn hook_refusal_text_names_the_hook_and_its_exit_and_leaves_out_its_output() {
    let rows: [(&'static str, Option<i32>, &str); 3] = [
        ("pre-commit", Some(1), "the pre-commit hook refused the commit with exit code 1"),
        ("commit-msg", Some(2), "the commit-msg hook refused the commit with exit code 2"),
        ("pre-commit", None, "the pre-commit hook was killed by a signal"),
    ];
    for (hook, code, expected) in rows {
        let refusal = HookRefusal { hook, code, stdout: b"lint: out\n".to_vec(), stderr: b"lint: err\n".to_vec() };
        assert_eq!(hook_refusal_text(&refusal), expected, "hook {hook}, code {code:?}");
    }
}
