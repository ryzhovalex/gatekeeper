use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    db::{self, Con, Id},
    quco::Collection,
    ryz::{enm::StrEnum, err, res::Res, time::{utc, Time}},
    schema, sql
};

#[derive(Serialize, Deserialize, Debug)]
pub enum ChangeAction {
    New,
    Del,
}

impl StrEnum for ChangeAction {
    fn to_str(&self) -> &str {
        match self {
            ChangeAction::New => "new",
            ChangeAction::Del => "del",
        }
    }

    fn from_str(s: &str) -> Res<Self> {
        match s {
            "new" => Ok(ChangeAction::New),
            "del" => Ok(ChangeAction::Del),
            _ => err::make_default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserChange {
    pub id: Id,
    pub created: Time,
    pub action: ChangeAction,
    pub user_id: Id,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = schema::user_change)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct UserChangeTable {
    pub id: Id,
    pub created: Time,
    pub action: String,
    pub user_id: Id,
}

impl Collection<UserChange> for UserChangeTable {
    fn to_msg(&self) -> UserChange {
        UserChange {
            id: self.id.to_owned(),
            created: self.created.to_owned(),
            action: ChangeAction::from_str(self.action.as_str()).unwrap(),
            user_id: self.user_id.to_owned(),
        }
    }
}

pub struct NewUserChange {
    pub user_id: Id,
    pub action: ChangeAction,
}

#[derive(Insertable)]
#[diesel(table_name=schema::user_change)]
struct InsertNewUserChange {
    pub user_id: Id,
    pub created: Time,
    pub action: String,
}

/// Fetches all user changes for a domain.
///
/// # Args
///
/// * `from` - from which time to fetch changes
pub fn get_many(from: Time, con: &mut Con) -> Res<Vec<UserChange>> {
    let user_changes = schema::user_change::table
        .filter(schema::user_change::created.ge(from))
        .select(UserChangeTable::as_select())
        .load(con)
        .unwrap();
    Ok(user_changes.iter().map(|x| x.to_msg()).collect())
}

pub fn new(data: &NewUserChange, con: &mut Con) -> Res<UserChange> {
    let change: UserChangeTable = diesel::insert_into(schema::user_change::table)
        .values(&InsertNewUserChange {
            user_id: data.user_id,
            created: utc(),
            action: data.action.to_str().to_string()
        })
        .returning(UserChangeTable::as_returning())
        .get_result(con)
        .unwrap();
    Ok(change.to_msg())
}
