use postgres::Client;
use serde::{Deserialize, Serialize};

use crate::{
    common::NULL, db, err::Err, password::hash_password, res::Res, Apprc, Reg,
};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub username: String,
    pub firstname: String,
    pub patronym: String,
    pub surname: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserChange {
    pub id: u32,
    pub action: String,
}

#[derive(Serialize, Deserialize)]
pub struct Domain {
    pub key: String,
    pub pending_user_changes: Vec<UserChange>,
}

pub fn create(data: &Reg, apprc: &Apprc) -> Res<User> {
    let mut con = db::con(&apprc.sql).unwrap();

    let hpassword = hash_password(&data.password).unwrap();
    con.execute(
        "
            INSERT INTO appuser (username, hpassword)
            VALUES ($1, $2)
        ",
        &[&data.username, &hpassword],
    )
    .unwrap();
    let row = con
        .query_one(
            "SELECT * FROM appuser WHERE username = $1",
            &[&data.username],
        )
        .unwrap();
    Ok(User {
        id: row.get("id"),
        username: row.get("username"),
        firstname: row.get("firstname"),
        patronym: row.get("patronym"),
        surname: row.get("surname"),
    })
}

pub fn get_by_username(username: &String) -> Res<User> {
    todo!()
}

pub fn set_rt_for_username(username: &String, rt: &String) -> Res<NULL> {
    todo!()
}

pub fn get_all_sids() -> Res<Vec<User>> {
    todo!()
}

pub fn get_domain_user_changes() -> Res<Vec<UserChange>> {
    todo!()
}
