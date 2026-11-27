//! What a request folder stands on: its record's base and partners read for a verb, the conversions between the
//! folders crate's RecordedBase and the join crate's RequestBase, the base a join op recorded before and after the
//! join, the partners' renderings as a report line and as JSON objects, and where a request forks (its recorded base
//! when cut with partners, else its merge base with the zone), for mr folder request, mr join, mr resolve, mr undo,
//! mr status, mr overlap and mr after-publish.

mod fork_point;
mod op_base;
mod partners_json;
mod partners_line;
mod read_request_base;
mod recorded_base_of;
mod request_base_of;

#[cfg(test)]
mod op_base_tests;
#[cfg(test)]
mod partners_json_tests;
#[cfg(test)]
mod partners_line_tests;
#[cfg(test)]
mod recorded_base_of_tests;
#[cfg(test)]
mod request_base_of_tests;

pub(crate) use fork_point::fork_point;
pub(crate) use op_base::op_base;
pub(crate) use partners_json::partners_json;
pub(crate) use partners_line::partners_line;
pub(crate) use read_request_base::read_request_base;
pub(crate) use recorded_base_of::recorded_base_of;
pub(crate) use request_base_of::request_base_of;
