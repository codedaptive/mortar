//! run_folder_request: `mr folder request <zone> <request> <path> --paths <path>...`, a request's folder made from
//! its zone's branch, holding only its paths and the root files, with the config's build dirs shared into it from
//! the zone folder.
//! No sibling test: it takes a real lock and runs git, the op log and folders; request_folder_paths_tests,
//! folder_setup_for_tests, folder_made_report_tests, build_output_lines_tests and build_output_json_tests cover its
//! decisions. The integration test crates/cli/tests/folder_runs/run_folder_request_tests.rs covers it through the
//! binary: the folder holding only its --paths and the root file, recorded as a request folder, its branch made by
//! an op of the actor, and, the zone having no zone folder, each build dir reported refused
//! (request_folder_holds_only_its_paths_and_the_root_files); a zone with no branch refused, nothing made; and the
//! build dirs shared, absent, kept on a repeat, and refused without failing the verb
//! (request_folder_build_output_shared, request_folder_build_output_absent,
//! request_folder_build_output_kept_on_repeat, request_folder_build_output_refused_without_failing).
//! Integration tests to write (not written here): a request branch already at another commit or
//! checked out elsewhere, the cli's why and next; a folders lock held by a live pid, exit 3; free disk under the floor, exit 1
//! with the cli's facts.

use std::path::Path;

use folders::{clone_build_dirs, make_request_folder, BuildDirClone};
use gitio::Git;
use oplog::{KillPidProbe, LockName};

use super::{
    build_output_json, build_output_lines, folder_error_report, folder_made_report, folder_setup_for, zone_folder_of,
};
use crate::config::Config;
use crate::context::{repository_error, repository_report};
use crate::lock::locked_act;
use crate::opkey::op_key;
use crate::report::{OutValue, Report};
use crate::shell::ShellContext;

const VERB: &str = "folder";

/// `path` is taken from `cwd` when relative. Under join-<zone> after recover (locked_act), keyed by --key or
/// op_key("folder-request", the folder): folders' make_request_folder with `paths` (the --paths, then the root files)
/// and the setup folder_setup_for gives for the actor; then, under the same lock and key, the config's build_dirs
/// cloned into the folder from zone_folder_of's zone folder (none tried when build_dirs is empty). When
/// zone_folder_of finds no zone folder, each configured dir is Refused with its why and nothing is copied. The report
/// is folder_made_report's with build_output_lines' lines after its own and a last field build_output
/// (build_output_json's objects, one per configured dir, `[]` when none is configured); its exit and outcome stay
/// "made" whatever the clone gave, since a build dir not shared never fails the verb.
pub fn run_folder_request(
    context: &ShellContext,
    config: &Config,
    cwd: &Path,
    zone: &str,
    request: &str,
    path: &Path,
    paths: &[String],
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
        let record = make_request_folder(&setup, zone, request, paths)
            .map_err(|error| folder_error_report("request", &error))?;
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
        report.lines.extend(build_output_lines(&clones));
        report.fields.push(("build_output".to_string(), OutValue::Raw(build_output_json(&clones))));
        Ok(report)
    })
    .unwrap_or_else(|report| report)
}
