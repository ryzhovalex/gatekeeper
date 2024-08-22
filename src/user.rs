use std::fmt::{Debug, Display};

use diesel::prelude::*;
use postgres::{Client, Row};
use serde::{Deserialize, Serialize};
use serde_json;

use crate::{
    asrt,
    db::{self, Con, Id},
    password::hash_password,
    quco::Collection,
    ryz::{
        asrt,
        err::{self, make, Error},
        query::Query,
        res::Res,
    },
    schema,
    user_change::{self, ChangeAction, NewUserChange},
    Apprc, InsertableReg, Reg,
};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: Id,
    pub username: String,
    pub firstname: Option<String>,
    pub patronym: Option<String>,
    pub surname: Option<String>,
    pub rt: Option<String>,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::user)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserTable {
    pub id: Id,
    pub hpassword: String,
    pub username: String,
    pub firstname: Option<String>,
    pub patronym: Option<String>,
    pub surname: Option<String>,
    pub rt: Option<String>,
}

impl Collection<User> for UserTable {
    fn to_msg(&self) -> User {
        User {
            id: self.id.to_owned(),
            username: self.username.to_owned(),
            firstname: self.firstname.to_owned(),
            patronym: self.patronym.to_owned(),
            surname: self.surname.to_owned(),
            rt: self.rt.to_owned(),
        }
    }
}

pub fn new(reg: &Reg, apprc: &Apprc, con: &mut Con) -> Res<User> {
    let hpassword = hash_password(&reg.password).unwrap();
    let user: UserTable = diesel::insert_into(schema::user::table)
        .values(&InsertableReg {
            username: reg.username.to_owned(),
            hpassword: hpassword.to_owned(),
            firstname: reg.firstname.to_owned(),
            patronym: reg.patronym.to_owned(),
            surname: reg.surname.to_owned(),
        })
        .returning(UserTable::as_returning())
        .get_result(con)
        .unwrap();

    user_change::new(
        &NewUserChange {
            user_id: user.id,
            action: ChangeAction::New,
        },
        apprc,
        con,
    )
    .unwrap();

    Ok(user.to_msg())
}

pub fn del_one(sq: &Query, con: &mut Con) -> Res<()> {
    let id = sq.get("id");
    let username = sq.get("username");
    let mut q = diesel::delete(schema::user::table).into_boxed();
    if id.is_some() {
        let val = serde_json::from_value::<Id>(id.unwrap().clone()).unwrap();
        q = q.filter(schema::user::id.eq(val));
    }
    if username.is_some() {
        let val = serde_json::from_value::<String>(username.unwrap().clone())
            .unwrap();
        q = q.filter(schema::user::username.eq(val));
    }

    let id = q.returning(schema::user::id).get_result::<Id>(con).unwrap();

    user_change::new(
        &NewUserChange {
            user_id: id,
            action: ChangeAction::Del,
        },
        con,
    )
    .unwrap();

    Ok(())
}

pub fn get_by_id(id: i32, con: &mut Con) -> Res<User> {
    Ok(schema::user::table
        .find(id)
        .select(UserTable::as_select())
        .first(con)
        .unwrap()
        .to_msg())
}

pub fn get_by_username_with_hpassword(
    username: &String,
    con: &mut Con
) -> Res<(User, String)> {

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

pub fn get_all_ids(apprc: &Apprc) -> Res<Vec<Id>> {
    let mut con = db::con(&apprc.sql).unwrap();
    let rows = con.query("SELECT id FROM appuser", &[]).unwrap();
    let mut ids: Vec<Id> = Vec::new();
    for r in rows {
        ids.push(r.get("id"));
    }
    Ok(ids)
}
