//! ZUTs for attempts_dir: relative dirs taken from the toplevel, lexical normalization, and the inside refusal.

use std::path::{Path, PathBuf};

use super::attempts_dir;

const TOP: &str = "/work/request";

fn resolved(dir: &str) -> Result<PathBuf, String> {
    attempts_dir(Path::new(TOP), Path::new(dir))
}

fn inside(dir: &str) -> String {
    format!("<dir> {dir} is inside the request folder /work/request; attempts go outside it")
}

#[test]
fn a_relative_dir_climbing_out_with_dot_dot_is_the_toplevels_sibling() {
    assert_eq!(resolved("../tries"), Ok(PathBuf::from("/work/tries")));
}

#[test]
fn a_relative_dir_climbing_two_levels_lands_above_the_parent() {
    assert_eq!(resolved("../../elsewhere/tries"), Ok(PathBuf::from("/elsewhere/tries")));
}

#[test]
fn a_relative_dir_inside_the_toplevel_is_refused() {
    assert_eq!(resolved("tries"), Err(inside("/work/request/tries")));
}

#[test]
fn dot_and_dot_dot_that_stay_inside_are_refused() {
    assert_eq!(resolved("./sub/../tries"), Err(inside("/work/request/tries")));
}

#[test]
fn dot_and_dot_dot_that_leave_are_accepted() {
    assert_eq!(resolved("sub/./../../tries/./x"), Ok(PathBuf::from("/work/tries/x")));
}

#[test]
fn a_relative_dir_that_leaves_and_comes_back_in_is_refused() {
    assert_eq!(resolved("../request/tries"), Err(inside("/work/request/tries")));
}

#[test]
fn a_sibling_whose_name_starts_with_the_toplevels_is_outside() {
    assert_eq!(resolved("../request-tries"), Ok(PathBuf::from("/work/request-tries")));
}

#[test]
fn an_absolute_dir_outside_is_kept() {
    assert_eq!(resolved("/tmp/tries"), Ok(PathBuf::from("/tmp/tries")));
}

#[test]
fn an_absolute_dir_is_normalized() {
    assert_eq!(resolved("/work/request/../tries/."), Ok(PathBuf::from("/work/tries")));
}

#[test]
fn an_absolute_dir_inside_is_refused() {
    assert_eq!(resolved("/work/request/src/tries"), Err(inside("/work/request/src/tries")));
}

#[test]
fn the_toplevel_itself_is_refused() {
    assert_eq!(resolved("."), Err(inside("/work/request")));
    assert_eq!(resolved("/work/request"), Err(inside("/work/request")));
    assert_eq!(resolved("src/.."), Err(inside("/work/request")));
}
