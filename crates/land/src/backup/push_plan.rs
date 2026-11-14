//! PushPlan: the one push a backup run makes. A declaration only (S-7); plan_backup_tests and backup_answer_tests
//! exercise it.

use gitio::PushRef;

/// `push`: each local refname with the PushRef that sets or deletes its backup ref, in push order.
/// `unchanged`: the local refnames whose backup already holds their value, which are not pushed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PushPlan {
    pub push: Vec<(String, PushRef)>,
    pub unchanged: Vec<String>,
}
