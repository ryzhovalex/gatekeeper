//! Operates over SQL queries.

use crate::rskit::res::Res;

pub fn build_where_in(target: &str, in_arr: &[String]) -> Res<String> {
    Ok(format!("WHERE {} IN ({})", target, in_arr.join(", ")))
}
