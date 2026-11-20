//! `mr install` and `mr uninstall`: placing the binary in `~/.local/bin`, the `mr` link, the
//! harness skill files, install's record, and uninstall by that record.

mod apply_failure;
mod apply_install;
mod apply_step;
mod binary_at;
mod choose_harnesses;
mod content_hash;
mod decode_record;
mod dir_position;
mod encode_record;
mod find_harness;
mod fresh_temp;
mod harness_spec;
mod harnesses;
mod home_unset_refusal;
mod install_env;
mod install_facts;
mod install_in;
mod install_item;
mod install_options;
mod install_paths;
mod install_paths_for;
mod install_plan;
mod install_report;
mod install_step;
mod item_json;
mod item_line;
mod item_state;
mod link_at;
mod link_plan;
mod make_dirs;
mod merge_record;
mod observe_item;
mod on_install_map;
mod package_prefixes;
mod parse_install_args;
mod parse_picker_answer;
mod path_facts;
mod placement_skip;
mod plan_install;
mod plan_link;
mod plan_placement;
mod plan_skills;
mod plan_uninstall;
mod random_token;
mod read_install_facts;
mod read_path_facts;
mod read_record;
mod record_item;
mod run_install;
mod run_uninstall;
mod skill_at;
mod skill_command;
mod skill_location;
mod skill_path;
mod skill_text;
mod step_record;
mod temp_path;
mod uninstall_failure;
mod uninstall_in;
mod uninstall_plan;
mod uninstall_report;
mod write_record;

#[cfg(test)]
mod integration;
#[cfg(test)]
mod apply_failure_tests;
#[cfg(test)]
mod choose_harnesses_tests;
#[cfg(test)]
mod content_hash_tests;
#[cfg(test)]
mod decode_record_tests;
#[cfg(test)]
mod dir_position_tests;
#[cfg(test)]
mod encode_record_tests;
#[cfg(test)]
mod find_harness_tests;
#[cfg(test)]
mod fresh_temp_tests;
#[cfg(test)]
mod harnesses_tests;
#[cfg(test)]
mod home_unset_refusal_tests;
#[cfg(test)]
mod install_paths_for_tests;
#[cfg(test)]
mod install_report_tests;
#[cfg(test)]
mod item_json_tests;
#[cfg(test)]
mod item_line_tests;
#[cfg(test)]
mod merge_record_tests;
#[cfg(test)]
mod on_install_map_tests;
#[cfg(test)]
mod parse_install_args_tests;
#[cfg(test)]
mod parse_picker_answer_tests;
#[cfg(test)]
mod placement_skip_tests;
#[cfg(test)]
mod plan_install_tests;
#[cfg(test)]
mod plan_link_tests;
#[cfg(test)]
mod plan_placement_tests;
#[cfg(test)]
mod plan_skills_tests;
#[cfg(test)]
mod plan_uninstall_tests;
#[cfg(test)]
mod skill_command_tests;
#[cfg(test)]
mod skill_path_tests;
#[cfg(test)]
mod skill_text_tests;
#[cfg(test)]
mod step_record_tests;
#[cfg(test)]
mod temp_path_tests;
#[cfg(test)]
mod uninstall_failure_tests;
#[cfg(test)]
mod uninstall_report_tests;

pub(crate) use self::{
    apply_failure::apply_failure, apply_install::apply_install, apply_step::apply_step, binary_at::BinaryAt,
    choose_harnesses::choose_harnesses, content_hash::content_hash, decode_record::decode_record,
    dir_position::dir_position, encode_record::encode_record, find_harness::find_harness, fresh_temp::fresh_temp,
    harness_spec::HarnessSpec, harnesses::HARNESSES, home_unset_refusal::home_unset_refusal, install_env::InstallEnv,
    install_facts::InstallFacts, install_in::install_in, install_item::InstallItem, install_options::InstallOptions,
    install_paths::InstallPaths, install_paths_for::install_paths_for, install_plan::InstallPlan,
    install_report::install_report, install_step::InstallStep, item_json::item_json, item_line::item_line,
    item_state::ItemState, link_at::LinkAt, link_plan::LinkPlan, make_dirs::make_dirs, merge_record::merge_record,
    observe_item::observe_item, on_install_map::on_install_map, package_prefixes::PACKAGE_PREFIXES,
    parse_install_args::parse_install_args, parse_picker_answer::parse_picker_answer, path_facts::PathFacts,
    placement_skip::placement_skip, plan_install::plan_install, plan_link::plan_link, plan_placement::plan_placement,
    plan_skills::plan_skills, plan_uninstall::plan_uninstall, random_token::random_token,
    read_install_facts::read_install_facts, read_path_facts::read_path_facts, read_record::read_record,
    record_item::RecordItem, run_install::run_install, run_uninstall::run_uninstall, skill_at::SkillAt,
    skill_command::skill_command, skill_location::SkillLocation, skill_path::skill_path, skill_text::skill_text,
    step_record::step_record, temp_path::temp_path, uninstall_failure::uninstall_failure, uninstall_in::uninstall_in,
    uninstall_plan::UninstallPlan, uninstall_report::uninstall_report, write_record::write_record,
};
