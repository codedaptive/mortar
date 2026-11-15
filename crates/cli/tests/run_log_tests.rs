//! run_log: a filter word log does not know, a second --ref and a --ref without a value are each log's usage (exit
//! 4, next log's usage line), returned before the op log or git is read. With one op in the log, plain log lists it
//! with an empty "records" field, and --json puts the log's own line in "records", byte for byte.
//! ZIT (S-79 as replaced: the line is the file): it calls log_filters and usage_report for real. The context is built
//! by hand. For the usage cases cwd does not exist, so a run that read git would answer a refusal or failure, not
//! exit 4. For the listing cases cwd is a throwaway repository (testkit's TestRepo, removed after the test): real git
//! answers its common dir, with GIT_CONFIG_GLOBAL=/dev/null and GIT_CONFIG_NOSYSTEM=1 set in this test process since
//! run_log builds its own git, and oplog reads a real op log file holding one line written here by hand.

use std::fs;
use std::path::Path;
use std::sync::Once;

use cli::{find_verb, run_log, ExitCode, OutValue, ParsedArgs, Report, ShellContext};
use testkit::TestRepo;

const LOG_USAGE: &str = "mortar log [--ref <full ref name>] [--actor <name>] [--key <operation key>] [--json]";

/// One op with no values and no remote mark, in oplog's key order.
const OP_1: &str = r#"{"op_id":"op-1","key":"k","kind":"save","actor":"core/fix-1","reason":"mortar save","at":5,"phase":"intent","moves":[{"ref":"refs/heads/req/core/fix-1","old":null,"new":"cccccccccccccccccccccccccccccccccccccccc"}]}"#;

fn context(rest: &[&str]) -> ShellContext {
    context_with(rest, false)
}

fn context_with(rest: &[&str], json: bool) -> ShellContext {
    ShellContext {
        args: ParsedArgs {
            verb: find_verb("log").expect("log is a verb"),
            rest: rest.iter().map(ToString::to_string).collect(),
            json,
            key: None,
            actor: None,
        },
        actor: String::new(),
        folder: None,
        config: None,
        protected: Vec::new(),
    }
}

fn assert_usage(rest: &[&str], reason: &str) {
    let report = run_log(&context(rest), Path::new("/nonexistent/mortar-run-log-zit"));
    assert_eq!(report.verb, "log");
    assert_eq!(report.exit, ExitCode::Usage);
    assert_eq!(report.outcome, "usage");
    assert_eq!(report.lines[0], reason);
    assert_eq!(report.next, LOG_USAGE);
    assert!(report.fields.is_empty());
}

#[test]
fn a_word_that_is_not_a_filter_is_logs_usage() {
    assert_usage(&["oops"], "unknown word \"oops\"");
}

#[test]
fn a_second_ref_is_logs_usage() {
    assert_usage(&["--ref", "refs/heads/a", "--ref=refs/heads/b"], "--ref is given twice");
}

#[test]
fn a_ref_without_a_value_is_logs_usage() {
    assert_usage(&["--ref"], "--ref needs a value");
}

static GIT_CONFIG: Once = Once::new();

/// run_log in a throwaway repository whose op log holds the one line OP_1.
fn log_of_one_op(json: bool) -> Report {
    GIT_CONFIG.call_once(|| {
        std::env::set_var("GIT_CONFIG_GLOBAL", "/dev/null");
        std::env::set_var("GIT_CONFIG_NOSYSTEM", "1");
    });
    let repo = TestRepo::new().expect("a throwaway repository");
    fs::create_dir(repo.path.join(".git/mortar")).expect("the mortar folder");
    fs::write(repo.path.join(".git/mortar/oplog.jsonl"), format!("{OP_1}\n")).expect("the op log");
    let report = run_log(&context_with(&[], json), &repo.path);
    repo.close().expect("the repository is removed");
    report
}

#[test]
fn plain_log_lists_the_op_with_no_records() {
    let report = log_of_one_op(false);
    assert_eq!(report.exit, ExitCode::Done);
    assert_eq!(report.outcome, "listed");
    assert_eq!(report.fields, vec![("records".to_string(), OutValue::Raw(Vec::new()))]);
    assert_eq!(
        report.lines,
        vec![
            "op-1 intent save by core/fix-1 at 5 key k: mortar save".to_string(),
            "  refs/heads/req/core/fix-1 absent -> cccccccccccccccccccccccccccccccccccccccc".to_string(),
        ]
    );
}

#[test]
fn json_log_holds_the_logs_own_line() {
    let report = log_of_one_op(true);
    assert_eq!(report.exit, ExitCode::Done);
    assert_eq!(report.outcome, "listed");
    assert_eq!(report.fields, vec![("records".to_string(), OutValue::Raw(vec![OP_1.to_string()]))]);
}
