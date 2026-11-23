//! run_folder_request: `mr folder request <zone> <request> <path> --paths <path>... [--with <zone>]...`, a request's
//! folder made from its zone's branch, or with partners from a merge of the zone's tip with each partner zone's tip,
//! holding only its paths and the root files, with the config's build dirs shared into it from the zone folder.
//! No sibling test: it takes a real lock and runs git, the op log, join and folders; request_folder_paths_tests,
//! folder_setup_for_tests, folder_made_report_tests, partner_conflict_report_tests, partners_line_tests,
//! partners_json_tests, build_output_lines_tests and build_output_json_tests cover its decisions. The integration test
//! crates/cli/tests/folder_runs/run_folder_request_tests.rs covers it through the binary: the folder holding only its
//! --paths and the root file, recorded as a request folder, its branch made by an op of the actor, and, the zone
//! having no zone folder, each build dir reported refused (request_folder_holds_only_its_paths_and_the_root_files);
//! a zone with no branch refused, nothing made; and the build dirs shared, absent, kept on a repeat, and refused
//! without failing the verb (request_folder_build_output_shared, request_folder_build_output_absent,
//! request_folder_build_output_kept_on_repeat, request_folder_build_output_refused_without_failing).
//! crates/cli/tests/folder_runs/partner_cut_tests.rs covers --with: the folder cut on the merge of zone and partner,
//! its partners reported and recorded, a repeat answering the same base (partner_cut_and_repeat); a partner the zone
//! already holds reported in partners_in_zone and not recorded (partner_cut_reports_partner_already_in_zone); a
//! conflicting merge refused with its partner and files, nothing made (partner_cut_conflict_makes_nothing).
//! Integration tests to write (not written here): a request branch already at another commit or
//! checked out elsewhere, the cli's why and next; a folders lock held by a live pid, exit 3; free disk under the floor, exit 1
//! with the cli's facts; a partner zone with no branch refused, nothing made.

use std::path::Path;

use folders::{clone_build_dirs, make_request_folder, read_recorded_base, BuildDirClone, RequestPartner};
use gitio::Git;
use oplog::{KillPidProbe, LockName};

use super::{
    build_output_json, build_output_lines, cut_on_partners, folder_error_report, folder_made_report,
    folder_setup_for, partners_json, partners_line, zone_folder_of,
};
use crate::config::Config;
use crate::context::{repository_error, repository_report};
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{failure_report, OutValue, Report};
use crate::shell::ShellContext;

const VERB: &str = "folder";

/// `path` is taken from `cwd` when relative. Under join-<zone> after recover (locked_act), with --key or
/// op_key("folder-request", the folder) as the op key locked_act records with the lock (a repeat is answered by
/// folders' own refill of the folder, not by the key): with no `partners`, folders' make_request_folder with `paths`
/// (the --paths, then the root files) and the setup folder_setup_for gives for the actor; with partners,
/// cut_on_partners' folder. Then, under the same lock and key, the config's build_dirs
/// cloned into the folder from zone_folder_of's zone folder (none tried when build_dirs is empty). When
/// zone_folder_of finds no zone folder, each configured dir is Refused with its why and nothing is copied. The report
/// is folder_made_report's with build_output_lines' lines after its own and a last field build_output
/// (build_output_json's objects, one per configured dir, `[]` when none is configured); its exit and outcome stay
/// "made" whatever the clone gave, since a build dir not shared never fails the verb. With partners, the partners
/// read back from the new folder's record (read_recorded_base) add partners_line's line right after
/// folder_made_report's lines and a field partners (partners_json's objects) after paths; an unreadable or missing
/// record is failure_report. A requested partner the record does not hold (join's merge drops a partner the zone
/// already holds) adds, only then, the line "already in zone <zone>, not recorded as partners: <p, q>" after the
/// partners line and a field partners_in_zone (those names, in the order given) after partners.
#[allow(clippy::too_many_arguments)] // the folder's facts and the shell's, each one the verb's distinct input
pub fn run_folder_request(
    context: &ShellContext,
    config: &Config,
    cwd: &Path,
    zone: &str,
    request: &str,
    path: &Path,
    paths: &[String],
    partners: &[String],
) -> Report {
    let git = Git::new(cwd);
    let common = match git.common_dir() {
        Ok(common) => common,
        Err(error) => return repository_report(VERB, &repository_error(cwd, error)),
    };
    let path = cwd.join(path);
    let key = op_key(context.args.key.as_deref(), "folder-request", &path, &[]);
    let lock = LockName::Join(zone.to_string());
    locked_act(&git, &common, &lock, &key, &KillPidProbe, VERB, || {
        let setup = folder_setup_for(&git, path.clone(), config, &context.actor);
        let record = if partners.is_empty() {
            make_request_folder(&setup, zone, request, paths).map_err(|error| folder_error_report("request", &error))?
        } else {
            cut_on_partners(&setup, zone, request, paths, partners)?
        };
        let clones = if config.build_dirs.is_empty() {
            Vec::new()
        } else {
            match zone_folder_of(&git, zone) {
                Ok(parent) => clone_build_dirs(&parent, &path, &config.build_dirs),
                Err(why) => config
                    .build_dirs
                    .iter()
                    .map(|dir| BuildDirClone::Refused { dir: dir.clone(), why: why.clone() })
                    .collect(),
            }
        };
        let mut report = folder_made_report("request", &path, &record);
        if !partners.is_empty() {
            let recorded = recorded_partners(&git, &path)?;
            report.lines.push(partners_line(&recorded));
            report.fields.push(("partners".to_string(), OutValue::Raw(partners_json(&recorded))));
            let in_zone: Vec<String> =
                partners.iter().filter(|name| !recorded.iter().any(|kept| &kept.zone == *name)).cloned().collect();
            if !in_zone.is_empty() {
                report.lines.push(format!("already in zone {zone}, not recorded as partners: {}", in_zone.join(", ")));
                report.fields.push(("partners_in_zone".to_string(), OutValue::Texts(in_zone)));
            }
        }
        report.lines.extend(build_output_lines(&clones));
        report.fields.push(("build_output".to_string(), OutValue::Raw(build_output_json(&clones))));
        Ok(report)
    })
    .unwrap_or_else(|report| report)
}

/// The partners the record of the folder at `path` holds, read with read_recorded_base from its git dir; the git
/// dir unreadable, the record unreadable or no record is failure_report naming the folder.
fn recorded_partners(git: &Git, path: &Path) -> Result<Vec<RequestPartner>, Report> {
    let folder = path.display();
    let unread = |why: String| failure_report(VERB, &format!("mr folder request: the record of {folder}: {why}"));
    let git_dir = git.git_dir(path).map_err(|error| unread(error.to_string()))?;
    let recorded = read_recorded_base(&git_dir).map_err(|error| unread(error.to_string()))?;
    recorded.map(|recorded| recorded.partners).ok_or_else(|| unread("no record".to_string()))
}
