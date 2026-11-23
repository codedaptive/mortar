//! What a request folder stands on: its record's base and partners read for a verb, and the conversions between the
//! folders crate's RecordedBase and the join crate's RequestBase, for mr folder request, mr join, mr resolve and
//! mr status.

mod read_request_base;
mod recorded_base_of;
mod request_base_of;

#[cfg(test)]
mod recorded_base_of_tests;
#[cfg(test)]
mod request_base_of_tests;

pub(crate) use read_request_base::read_request_base;
pub(crate) use recorded_base_of::recorded_base_of;
pub(crate) use request_base_of::request_base_of;
