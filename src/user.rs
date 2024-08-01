use postgres::{Client, Row};
use serde::{Deserialize, Serialize};

use crate::{db, err::Err, password::hash_password, res::Res, Apprc, Reg};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub username: String,
    pub firstname: Option<String>,
    pub patronym: Option<String>,
    pub surname: Option<String>,
    pub rt: Option<String>,
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
            INSERT INTO appuser (username, hpassword, firstname, patronym, surname)
            VALUES ($1, $2, $3, $4, $5)
        ",
        &[
            &data.username,
            &hpassword,
            &data.firstname.to_owned().unwrap_or("".to_string()),
            &data.patronym.to_owned().unwrap_or("".to_string()),
            &data.surname.to_owned().unwrap_or("".to_string())
        ],
    )
    .unwrap();
    let row = con
        .query_one(
            "SELECT * FROM appuser WHERE username = $1",
            &[&data.username],
        )
        .unwrap();
    parse_row(&row)
}

fn parse_row(row: &Row) -> Res<User> {
    Ok(User {
        id: row.get::<_, i32>("id") as u32,
        username: row.get("username"),
        firstname: row.get("firstname"),
        patronym: row.get("patronym"),
        surname: row.get("surname"),
        rt: row.get("rt"),
    })
}

pub fn get_by_username(
    username: &String,
    apprc: &Apprc,
) -> Res<(User, String)> {
    let mut con = db::con(&apprc.sql).unwrap();

    let row = con
        .query_one("SELECT * FROM appuser WHERE username = $1", &[&username])
        .unwrap();

    Ok((parse_row(&row).unwrap(), row.get("hpassword")))
}

pub fn get_by_rt(rt: &String, apprc: &Apprc) -> Res<(User, String)> {
    let mut con = db::con(&apprc.sql).unwrap();

    let row = con
        .query_one("SELECT * FROM appuser WHERE rt = $1", &[&rt])
        .unwrap();

    Ok((parse_row(&row).unwrap(), row.get("hpassword")))
}

pub fn del_rt(rt: &String, apprc: &Apprc) -> Res<()> {
    let mut con = db::con(&apprc.sql).unwrap();

    con.query_one("UPDATE appuser SET rt = NULL WHERE rt = $1", &[&rt])
        .unwrap();

    Ok(())
}

pub fn set_rt_for_username(
    username: &String,
    rt: &String,
    apprc: &Apprc,
) -> Res<()> {
    let mut con = db::con(&apprc.sql).unwrap();

    con.query_one("UPDATE appuser SET rt = $1 WHERE username = $2", &[&rt, &username])
        .unwrap();

    Ok(())
}

pub fn get_all_sids(apprc: &Apprc) -> Res<Vec<User>> {
    todo!()
}

pub fn get_domain_user_changes(apprc: &Apprc) -> Res<Vec<UserChange>> {
    todo!()
}
