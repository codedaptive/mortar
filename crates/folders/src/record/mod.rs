//! The folder record: its type, its one-line JSON form, and where it lives.

mod decode_folder_record;
mod decode_record_and_partners;
mod encode_folder_record;
mod encode_record_and_partners;
mod folder_kind;
mod folder_record;
mod parse_record_json;
mod record_error;
mod record_error_display;
mod record_json_value;
mod record_partners;
mod record_path;
mod recorded_base;
mod request_partner;

#[cfg(test)]
mod decode_record_and_partners_tests;
#[cfg(test)]
mod encode_record_and_partners_tests;
#[cfg(test)]
mod decode_folder_record_tests;
#[cfg(test)]
mod encode_folder_record_tests;
#[cfg(test)]
mod parse_record_json_tests;
#[cfg(test)]
mod record_error_display_tests;
#[cfg(test)]
mod record_path_tests;

pub use decode_folder_record::decode_folder_record;
pub(crate) use decode_record_and_partners::decode_record_and_partners;
pub(crate) use encode_folder_record::encode_folder_record;
pub(crate) use encode_record_and_partners::encode_record_and_partners;
pub use folder_kind::FolderKind;
pub use folder_record::FolderRecord;
pub use record_error::RecordError;
pub use record_path::record_path;
pub use recorded_base::RecordedBase;
pub use request_partner::RequestPartner;

pub(crate) use oplog::JsonError;
pub(crate) use parse_record_json::parse_record_json;
pub(crate) use record_json_value::RecordJsonValue;
