//! landing: land a squash on integration, commit named paths of the integration folder onto it (commit_files), back
//! it out and publish it, with the landing state `<common>/mortar/landing.json` that land and commit_files write
//! before their ops and back_out and publish read, and would_merge_with (a squash and further squashes merged in order
//! onto integration, as successive lands would).
//! Past 150 lines: this is the folder's bridge, one line per file it declares or exports.

mod advance_origin;
mod back_out;
mod back_out_answer;
mod back_out_onto;
mod back_out_step;
mod commit_files;
mod commit_files_answer;
mod commit_files_error;
mod commit_files_error_display;
mod commit_files_error_source;
mod decide_back_out;
mod decide_commit_files;
mod decide_landing;
mod decide_publish;
mod decide_refused_push;
mod decode_landing_state;
mod encode_landing_state;
mod fetch_origin;
mod finish_publish;
mod integration_op;
mod is_fast_forward;
mod land;
mod land_answer;
mod land_error;
mod land_error_display;
mod land_error_source;
mod landing_applied;
mod landing_message;
mod landing_state;
mod landing_state_error;
mod landing_state_error_display;
mod landing_state_error_source;
mod landing_state_path;
mod landing_step;
mod landing_target;
mod landing_target_for;
mod make_landing_target;
mod publish;
mod publish_answer;
mod publish_error;
mod publish_error_display;
mod publish_error_source;
mod publish_outcome;
mod publish_push_op;
mod publish_step;
mod publish_with_push;
mod push_intent_close;
mod push_intent_state;
mod push_landed;
mod push_landing;
mod push_refusal;
mod push_result;
mod read_landing_state;
mod read_push_intent;
mod remote_origin;
mod remove_landing_state;
mod resolve_landing_state;
mod settle_push_intent;
mod would_merge_step;
mod would_merge_with;
mod would_merge_with_answer;
mod write_landing_state;

#[cfg(test)]
mod commit_files_error_display_tests;
#[cfg(test)]
mod commit_files_error_source_tests;
#[cfg(test)]
mod decide_back_out_tests;
#[cfg(test)]
mod decide_commit_files_tests;
#[cfg(test)]
mod decide_landing_tests;
#[cfg(test)]
mod decide_publish_tests;
#[cfg(test)]
mod decide_refused_push_tests;
#[cfg(test)]
mod is_fast_forward_tests;
#[cfg(test)]
mod land_error_display_tests;
#[cfg(test)]
mod land_error_source_tests;
#[cfg(test)]
mod landing_applied_tests;
#[cfg(test)]
mod landing_state_error_display_tests;
#[cfg(test)]
mod landing_state_error_source_tests;
#[cfg(test)]
mod landing_state_path_tests;
#[cfg(test)]
mod publish_error_display_tests;
#[cfg(test)]
mod publish_error_source_tests;
#[cfg(test)]
mod push_intent_close_tests;
#[cfg(test)]
mod push_refusal_tests;
#[cfg(test)]
mod resolve_landing_state_tests;
#[cfg(test)]
mod would_merge_step_tests;

pub use back_out::back_out;
pub use back_out_answer::BackOutAnswer;
pub use commit_files::commit_files;
pub use commit_files_answer::CommitFilesAnswer;
pub use commit_files_error::CommitFilesError;
pub use land::land;
pub use land_answer::LandAnswer;
pub use land_error::LandError;
pub use landing_state_error::LandingStateError;
pub use publish::publish;
pub use publish_answer::PublishAnswer;
pub use publish_error::PublishError;
pub use publish_outcome::PublishOutcome;
pub use publish_with_push::publish_with_push;
pub use would_merge_with::would_merge_with;
pub use would_merge_with_answer::WouldMergeWith;

pub(crate) use advance_origin::advance_origin;
pub(crate) use back_out_onto::back_out_onto;
pub(crate) use back_out_step::BackOutStep;
pub(crate) use decide_back_out::decide_back_out;
pub(crate) use decide_commit_files::decide_commit_files;
pub(crate) use decide_landing::decide_landing;
pub(crate) use decide_publish::decide_publish;
pub(crate) use decide_refused_push::decide_refused_push;
pub(crate) use decode_landing_state::decode_landing_state;
pub(crate) use encode_landing_state::encode_landing_state;
pub(crate) use fetch_origin::fetch_origin;
pub(crate) use finish_publish::finish_publish;
pub(crate) use integration_op::integration_op;
pub(crate) use is_fast_forward::is_fast_forward;
pub(crate) use landing_applied::landing_applied;
pub(crate) use landing_message::landing_message;
pub(crate) use landing_state::LandingState;
pub(crate) use landing_state_path::landing_state_path;
pub(crate) use landing_step::LandingStep;
pub(crate) use landing_target::LandingTarget;
pub(crate) use landing_target_for::landing_target_for;
pub(crate) use make_landing_target::make_landing_target;
pub(crate) use publish_push_op::publish_push_op;
pub(crate) use publish_step::PublishStep;
pub(crate) use push_intent_close::push_intent_close;
pub(crate) use push_intent_state::PushIntentState;
pub(crate) use push_landed::push_landed;
pub(crate) use push_landing::push_landing;
pub(crate) use push_refusal::push_refusal;
pub(crate) use push_result::PushResult;
pub(crate) use read_landing_state::read_landing_state;
pub(crate) use read_push_intent::read_push_intent;
pub(crate) use remote_origin::remote_origin;
pub(crate) use remove_landing_state::remove_landing_state;
pub(crate) use resolve_landing_state::resolve_landing_state;
pub(crate) use settle_push_intent::settle_push_intent;
pub(crate) use would_merge_step::would_merge_step;
pub(crate) use write_landing_state::write_landing_state;
