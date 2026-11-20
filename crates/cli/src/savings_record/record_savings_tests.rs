//! record_savings has no unit test: it reads the environment, the clock and the user's config file, and writes the
//! savings log. The integration tests in tests/status_need_runs/savings.rs cover it: the line and log on plain and
//! --json status, the switch off in ~/.config and in $XDG_CONFIG_HOME, an unparsable config leaving the line on,
//! $XDG_STATE_HOME over ~/.local/state, and no log on a usage error, a save or mr log
//! (savings_usage_refusal_and_act_have_no_metadata_or_log).
