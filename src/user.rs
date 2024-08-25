use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::{
    db::{Con, Id},
    password::hash_password,
    quco::Collection,
    ryz::{err, query::Query, res::Res},
    schema,
    user_change::{self, ChangeAction, NewUserChange},
    InsertReg, Reg,
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
#[diesel(table_name = schema::appuser)]
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
    if reg.username.starts_with("archive::") {
        return err::res_msg("cannot accept archived usernames");
    }
    let hpassword = hash_password(&reg.password).unwrap();
    let user: UserTable = diesel::insert_into(schema::appuser::table)
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

/// Instead of deletion, users are archived, their usernames are changed to
/// be archive::<username> and they are no more accessible. This needs to be
/// done due to user_change synchronization needs, the changes will still point
/// to the archived user.
pub fn del(sq: &Query, con: &mut Con) -> Res<()> {
    let id = sq.get("id");
    let username = sq.get("username");
    let mut q = schema::appuser::table.into_boxed();
    if id.is_some() {
        let val = serde_json::from_value::<Id>(id.unwrap().clone()).unwrap();
        q = q.filter(schema::appuser::id.eq(val));
    }
    if username.is_some() {
        let username =
            serde_json::from_value::<String>(username.unwrap().clone())
                .unwrap();
        if username.starts_with("archive::") {
            return err::res_msg("cannot accept archived usernames");
        }
        q = q.filter(schema::appuser::username.eq(username));
    }

    let username = q
        .select(schema::appuser::username)
        .get_result::<String>(con)
        .unwrap();
    let archived_username = "archived::".to_string() + username.as_str();

    let id = diesel::update(schema::appuser::table)
        .filter(schema::appuser::username.eq(username))
        .set(schema::appuser::username.eq(archived_username))
        .returning(schema::appuser::id)
        .get_result::<Id>(con)
        .unwrap();

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
    Ok(schema::appuser::table
        .filter(schema::appuser::id.eq(id))
        .filter(schema::appuser::username.not_like("archived::%"))
        .select(UserTable::as_select())
        .first(con)
        .unwrap()
        .to_msg())
}

pub fn get_by_username(
    username: &String,
    con: &mut Con,
) -> Res<(User, String)> {
    if username.starts_with("archive::") {
        return err::res_msg("cannot accept archived usernames");
    }
    let user: UserTable = schema::appuser::table
        .filter(schema::appuser::username.eq(username))
        .select(UserTable::as_select())
        .first(con)
        .unwrap();
    Ok((user.to_msg(), user.hpassword))
}

pub fn get_by_rt(rt: &String, con: &mut Con) -> Res<(User, String)> {
    let user: UserTable = schema::appuser::table
        .filter(schema::appuser::rt.eq(rt))
        .filter(schema::appuser::username.not_like("archived::%"))
        .select(UserTable::as_select())
        .first(con)
        .unwrap();
    Ok((user.to_msg(), user.hpassword))
}

pub fn del_rt(rt: &String, con: &mut Con) -> Res<()> {
    // here we can delete rt even for archived users
    diesel::update(schema::appuser::table.filter(schema::appuser::rt.eq(rt)))
        .set(schema::appuser::rt.eq::<Option<String>>(None))
        .execute(con)
        .unwrap();
    Ok(())
}

pub fn set_rt_for_username(
    username: &String,
    rt: &String,
    con: &mut Con,
) -> Res<()> {
    if username.starts_with("archive::") {
        return err::res_msg("cannot accept archived usernames");
    }
    diesel::update(
        schema::appuser::table.filter(schema::appuser::username.eq(username)),
    )
    .set(schema::appuser::rt.eq::<Option<String>>(Some(rt.to_owned())))
    .execute(con)
    .unwrap();
    Ok(())
}

pub fn get_many_as_ids(con: &mut Con) -> Res<Vec<Id>> {
    let ids = schema::appuser::table
        .filter(schema::appuser::username.not_like("archived::%"))
        .select(schema::appuser::id)
        .get_results::<Id>(con)
        .unwrap();
    Ok(ids)
}
