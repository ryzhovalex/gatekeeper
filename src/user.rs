use std::{
    any::Any,
    collections::HashMap,
    fmt::{Debug, Display},
};

use postgres::{Client, Row};
use serde::{Deserialize, Serialize};
use serde_json;

use crate::{
    db,
    password::hash_password,
    rskit::{
        err::{err, ErrData},
        query::Query,
        res::Res,
    },
    Apprc, Reg,
};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub firstname: Option<String>,
    pub patronym: Option<String>,
    pub surname: Option<String>,
    pub rt: Option<String>,
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

pub fn del(searchq: &Query, apprc: &Apprc) -> Res<()> {
    let mut con = db::con(&apprc.sql).unwrap();

    let id = searchq.get("id");
    let username = searchq.get("username");
    let where_ = if id.is_some() && username.is_some() {
        format!(
            "id = {} AND username = '{}'",
            id.unwrap(),
            username.unwrap().as_str().unwrap()
        )
    } else if id.is_some() {
        format!("id = {}", id.unwrap())
    } else if username.is_some() {
        format!("username = '{}'", username.unwrap().as_str().unwrap())
    } else {
        String::new()
    };

    if where_.is_empty() {
        return err(
            "val_err",
            format!("failed to process searchq {:?}", searchq),
        );
    }

    let stmt = format!("DELETE FROM appuser WHERE {}", &where_);
    dbg!(&stmt);
    con.batch_execute(stmt.as_str()).unwrap();

    Ok(())
}

fn parse_row(row: &Row) -> Res<User> {
    Ok(User {
        id: row.get("id"),
        username: row.get("username"),
        firstname: row.get("firstname"),
        patronym: row.get("patronym"),
        surname: row.get("surname"),
        rt: row.get("rt"),
    })
}

pub fn get_by_id(id: i32, apprc: &Apprc) -> Res<User> {
    let mut con = db::con(&apprc.sql).unwrap();

    let row = con
        .query_one("SELECT * FROM appuser WHERE id = $1", &[&id])
        .unwrap();

    Ok(parse_row(&row).unwrap())
}

pub fn get_by_username_with_hpassword(
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

    con.execute("UPDATE appuser SET rt = NULL WHERE rt = $1", &[&rt])
        .unwrap();

    Ok(())
}

pub fn set_rt_for_username(
    username: &String,
    rt: &String,
    apprc: &Apprc,
) -> Res<()> {
    let mut con = db::con(&apprc.sql).unwrap();

    con.execute(
        "UPDATE appuser SET rt = $1 WHERE username = $2",
        &[&rt, &username],
    )
    .unwrap();

    Ok(())
}

pub fn get_all_sids(apprc: &Apprc) -> Res<Vec<User>> {
    todo!()
}
