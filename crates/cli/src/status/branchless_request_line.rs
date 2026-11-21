//! branchless_request_line: the cli's words for a request whose branch name the plan refuses, shared by
//! read_branch_reads (mr status, mr disk) and read_branch_tips (mr need) so both say the same thing.
//! branchless_request_line_tests covers it.

/// "request <request> in zone <zone> is not a name a branch can carry".
pub(crate) fn branchless_request_line(request: &str, zone: &str) -> String {
    format!("request {request} in zone {zone} is not a name a branch can carry")
}
