use crate::RunOpts;

#[test]
fn default_is_all_empty_and_not_read_only() {
    let opts = RunOpts::default();
    assert_eq!(opts.stdin, None);
    assert!(opts.env.is_empty());
    assert_eq!(opts.cwd, None);
    assert!(opts.answer_codes.is_empty());
    assert!(!opts.read_only);
}
