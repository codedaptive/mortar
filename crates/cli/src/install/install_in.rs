//! install_in: `mr install` against a given environment: reads, plans, then writes and records (spec 07 item 7).
//! Every refusal comes before the first write: an unreadable record, the harness choice (--target, the picker),
//! the plan's refusals (placement, link, skills), and a path the record cannot hold (the record's own path
//! included). A plan with no step writes nothing at all. Otherwise the first write is the record's folder: if it
//! cannot be made, the folders made for it are removed again and nothing is placed. Then the steps are applied, and
//! whenever any folder was made or any step applied, the record is written last, also after a step failed, so
//! uninstall can still find what was placed. A step's failure then names each recorded path (apply_failure): a
//! repeat finishes the install, or uninstall, named "mr" only if the plan chose it and its link was applied, else
//! "mortar", removes them. Apart from folders the
//! failure path could not remove (named in its report), anything left unrecorded is only when writing the record
//! itself fails; that failure lists each such path.
//! No sibling test: it touches the disk; the install ZITs in install/integration/ cover it.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::report::{failure_report, ExitCode, Report};

use super::{
    apply_failure, apply_install, choose_harnesses, encode_record, install_paths_for, install_report, make_dirs,
    merge_record, placement_skip, plan_install, random_token, read_install_facts, read_record, step_record,
    write_record, InstallEnv, InstallFacts, InstallOptions, InstallPaths, InstallPlan, RecordItem,
};

/// The report: a refusal (exit 1) or usage report (exit 4) before anything is written, a failure (exit 5) when the
/// disk fails, or install's report. `ask` asks the picker's question; it is called only on a terminal with neither
/// --target nor -y. Placement is skipped when this binary's canonical path is under a prefix as canonicalized (a
/// prefix that cannot be canonicalized is taken as given), so a prefix reached through a symlink still counts.
pub fn install_in(env: &InstallEnv, options: &InstallOptions, ask: &mut dyn FnMut(&str) -> Option<String>) -> Report {
    let paths = install_paths_for(&env.home);
    let old = match read_record(&paths.record) {
        Ok(old) => old.unwrap_or_default(),
        Err(reason) => return refusal(reason, format!("fix or remove {}", paths.record.display())),
    };
    let facts = read_install_facts(env, &paths, options.location, old);
    let prefixes: Vec<_> =
        env.prefixes.iter().map(|prefix| fs::canonicalize(prefix).unwrap_or_else(|_| prefix.clone())).collect();
    let skip = placement_skip(facts.this.as_deref(), &prefixes, options.no_place);
    let planned = choose_harnesses(&facts.detected, options, env.terminal, ask)
        .and_then(|(chosen, note)| plan_install(&facts, options, &paths, skip, &chosen, note));
    let plan = match planned {
        Ok(plan) => plan,
        Err(report) => return report,
    };
    let hash = facts.this_hash.clone().unwrap_or_default();
    let records: Vec<_> = plan.steps.iter().map(|step| step_record(step, &hash)).collect();
    // The record's path holds the state folders' paths too, so it stands for every Dir item install may add for them.
    let recordable = encode_record(&merge_record(&facts.record, &records))
        .and_then(|_| paths.record.to_str().map(drop).ok_or_else(|| paths.record.clone()));
    if let Err(path) = recordable {
        return refusal(
            format!("{} is not UTF-8, so install cannot record it", path.display()),
            "install with a HOME, and for --location local a project, whose paths are UTF-8".to_string(),
        );
    }
    if plan.steps.is_empty() {
        return install_report(&plan);
    }
    write_install(&plan, &facts, &paths, &hash)
}

/// Makes the record's folder first (on failure, undoes it and places nothing), applies the plan's steps, then writes
/// the record: the earlier record's items with the made folders and applied items merged in; with no such items,
/// nothing is written. On a failure (exit 5): a step failed and the record did not fail, apply_failure, which with
/// items lists them (the record was written) for uninstall (named as apply_failure does), and with none says
/// nothing was applied or made, so nothing was recorded; the record could not be written, unrecorded_failure with
/// the step's error (if one failed) and then the record's. Else install's report.
fn write_install(plan: &InstallPlan, facts: &InstallFacts, paths: &InstallPaths, hash: &str) -> Report {
    let (made, state_dir) = make_dirs(&paths.state_dir);
    if let Err(error) = state_dir {
        return state_dir_failure(&paths.state_dir, &error, &made);
    }
    let (applied, result) = apply_install(&plan.steps, hash);
    let items: Vec<RecordItem> = made.into_iter().map(|path| RecordItem::Dir { path }).chain(applied).collect();
    let recorded = if items.is_empty() {
        Ok(())
    } else {
        encode_record(&merge_record(&facts.record, &items))
            .map_err(|path| format!("{} is not UTF-8, so install cannot record it", path.display()))
            .and_then(|text| {
                write_record(&paths.record, &text, &mut random_token)
                    .map_err(|error| format!("cannot write {}: {error}", paths.record.display()))
            })
    };
    match (result, recorded) {
        (Ok(()), Ok(())) => install_report(plan),
        (Err(error), Ok(())) => apply_failure(&error, &items, plan.command),
        (result, Err(record_error)) => {
            unrecorded_failure(result.err().into_iter().chain([record_error]).collect(), &items)
        }
    }
}

/// The record's folder could not be made: the folders made for it are removed again, deepest first (they are empty
/// and install's own), and any that would not go is named instead of "nothing half-done is left".
fn state_dir_failure(state_dir: &Path, error: &io::Error, made: &[PathBuf]) -> Report {
    let mut report = failure_report("install", &format!("cannot make {}: {error}", state_dir.display()));
    let stuck: Vec<String> = made
        .iter()
        .rev()
        .filter_map(|dir| fs::remove_dir(dir).err().map(|why| format!("left {}: {why}", dir.display())))
        .collect();
    if !stuck.is_empty() {
        report.lines.truncate(1);
        report.lines.extend(stuck);
    }
    report
}

/// The record could not be written, so what was placed and made is not in it: the errors, then each such path for
/// a person to remove, since uninstall cannot find them.
fn unrecorded_failure(errors: Vec<String>, items: &[RecordItem]) -> Report {
    let mut report = failure_report("install", "");
    report.lines = errors;
    report.lines.extend(items.iter().map(|item| {
        let (RecordItem::File { path, .. } | RecordItem::Link { path, .. } | RecordItem::Dir { path }) = item;
        format!("not recorded, so uninstall cannot remove it: {}", path.display())
    }));
    report.next = "remove the paths listed, then repeat the same command".to_string();
    report
}

/// A refusal before anything is written: `reason`, that nothing changed, and `next`.
fn refusal(reason: String, next: String) -> Report {
    Report {
        verb: "install".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![reason, "nothing changed".to_string()],
        owner: Some("the person installing Mortar".to_string()),
        next,
        fields: Vec::new(),
    }
}
