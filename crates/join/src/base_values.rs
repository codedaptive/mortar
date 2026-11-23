//! base_values: the two values a join op records for the request's base, before and after the join.
//! base_values_tests covers the two pairs and their order.

use crate::base_after_name::BASE_AFTER;
use crate::base_before_name::BASE_BEFORE;
use crate::encode_request_base::encode_request_base;
use crate::request_base::RequestBase;

/// BASE_BEFORE with `before`'s text, then BASE_AFTER with `after`'s, each as encode_request_base writes it.
pub(crate) fn base_values(before: &RequestBase, after: &RequestBase) -> Vec<(String, String)> {
    vec![
        (BASE_BEFORE.to_string(), encode_request_base(before)),
        (BASE_AFTER.to_string(), encode_request_base(after)),
    ]
}
