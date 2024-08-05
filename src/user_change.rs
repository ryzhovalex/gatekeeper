use postgres::Row;
use serde::{Deserialize, Serialize};

use crate::{db, rskit::res::Res, Apprc};

#[derive(Serialize, Deserialize)]
pub struct UserChange {
    pub id: i32,
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

pub fn get_domain_user_changes(apprc: &Apprc) -> Res<Vec<UserChange>> {
    todo!()
}

pub fn parse_user_change_row(row: &Row) {}

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
