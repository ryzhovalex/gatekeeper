//! Operates over SQL queries.

use crate::ryz::res::Res;

pub fn build_where_in(target: &str, in_arr: &[String]) -> Res<String> {
    Ok(format!("{} IN ({})", target, in_arr.join(", ")))
}
