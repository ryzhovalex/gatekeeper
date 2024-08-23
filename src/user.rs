use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::{
    db::{Con, Id},
    password::hash_password,
    quco::Collection,
    ryz::{
        query::Query,
        res::Res,
    },
    schema,
    user_change::{self, ChangeAction, NewUserChange}, InsertReg, Reg,
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

pub fn new(reg: &Reg, con: &mut Con) -> Res<User> {
    let hpassword = hash_password(&reg.password).unwrap();
    let user: UserTable = diesel::insert_into(schema::user::table)
        .values(&InsertReg {
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
        con,
    )
    .unwrap();

    Ok(user.to_msg())
}

pub fn del(sq: &Query, con: &mut Con) -> Res<()> {
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

pub fn get_by_username(
    username: &String,
    con: &mut Con,
) -> Res<(User, String)> {
    let user: UserTable = schema::user::table
        .filter(schema::user::username.eq(username))
        .select(UserTable::as_select())
        .first(con)
        .unwrap();
    Ok((user.to_msg(), user.hpassword))
}

pub fn get_by_rt(rt: &String, con: &mut Con) -> Res<(User, String)> {
    let user: UserTable = schema::user::table
        .filter(schema::user::rt.eq(rt))
        .select(UserTable::as_select())
        .first(con)
        .unwrap();
    Ok((user.to_msg(), user.hpassword))
}

pub fn del_rt(rt: &String, con: &mut Con) -> Res<()> {
    diesel::update(schema::user::table.filter(schema::user::rt.eq(rt)))
        .set(schema::user::rt.eq::<Option<String>>(None))
        .execute(con)
        .unwrap();
    Ok(())
}

pub fn set_rt_for_username(
    username: &String,
    rt: &String,
    con: &mut Con,
) -> Res<()> {
    diesel::update(
        schema::user::table.filter(schema::user::username.eq(username)),
    )
    .set(schema::user::rt.eq::<Option<String>>(Some(rt.to_owned())))
    .execute(con)
    .unwrap();
    Ok(())
}

pub fn get_many_as_ids(con: &mut Con) -> Res<Vec<Id>> {
    let ids = schema::user::table
        .select(schema::user::id)
        .get_results::<Id>(con)
        .unwrap();
    Ok(ids)
}
