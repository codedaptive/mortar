//! append_savings_log has no unit test: it makes a folder and writes a file. The integration tests in
//! tests/status_need_runs/savings.rs cover it: savings_home_log_appends_plain_and_json_answers (the folder is made
//! and each answer appends one line), savings_xdg_state_takes_precedence_over_home, and
//! savings_unwritable_state_does_not_change_answer (a failed write prints nothing and exits 0).
