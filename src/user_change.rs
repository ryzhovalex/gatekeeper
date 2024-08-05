use postgres::Row;
use serde::{Deserialize, Serialize};

use crate::{db::{self, Id}, rskit::res::Res, Apprc};

#[derive(Serialize, Deserialize)]
pub struct UserChange {
    pub id: Id,
    pub action: String,
}

pub struct CreateUserChange {
    pub username: String,
    pub action: String,
}

pub fn parse_row(row: &Row) -> Res<UserChange> {
    Ok(UserChange {
        id: row.get("id"),
        action: row.get("action"),
    })
}

/// Fetches all user changes for a domain.
///
/// # Args
///
/// * `domain_key` - Key of the domain for which to fetch changes.
/// * `unlink` - Whether to unlink changes for the requested domain.
///              Defaults to `true`.
pub fn get_domain_user_changes(
    domain_key: String,
    unlink: Option<bool>,
    apprc: &Apprc,
) -> Res<Vec<UserChange>> {
    let mut con = db::con(&apprc.sql).unwrap();
    let rows = con
        .query(
            "
            SELECT * FROM domain_to_user_changes
            JOIN domain ON domain.id = domain_to_user_changes.domain_id
            JOIN user_change ON user_change.id = domain_to_user_changes.user_change_id
            WHERE domain.key = $1",
            &[&domain_key],
        )
        .unwrap();
    let mut user_changes: Vec<UserChange> = Vec::new();
    let mut user_change_ids: Vec<Id> = Vec::new();
    for r in rows {
        user_changes.push(parse_row(&r).unwrap());
    }

    let unlink = match unlink {
        Some(v) => v,
        None => true
    };
    if unlink {
    }

    Ok(user_changes)
}

pub fn create(data: &CreateUserChange, apprc: &Apprc) -> Res<UserChange> {
    let mut con = db::con(&apprc.sql).unwrap();
    let row = con
        .query_one(
            "
            INSERT INTO user_change (user_id, action)
            VALUES ((SELECT id FROM appuser WHERE username = $1), $2)
            RETURNING *",
            &[&data.username, &data.action],
        )
        .unwrap();
    parse_row(&row)
}
