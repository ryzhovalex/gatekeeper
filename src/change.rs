use std::fmt::Write;

use postgres::{
    fallible_iterator::{FallibleIterator, Map},
    Client, Row,
};
use serde::{de::value, Deserialize, Serialize};

use crate::{
    asrt,
    db::{self, Id},
    ryz::{err, res::Res, time::Time},
    sql, Apprc,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Changes {
    user: Vec<UserChange>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserChange {
    pub id: Id,
    pub action: String,
    pub user_id: Id,
}

pub struct CreateUserChange {
    pub user_id: Option<Id>,
    pub username: Option<String>,
    pub action: String,
}

pub fn parse_row(row: &Row) -> Res<UserChange> {
    Ok(UserChange {
        id: row.get("id"),
        action: row.get("action"),
        user_id: row.get("user_detached_id"),
    })
}

/// Fetches all user changes for a domain.
///
/// # Args
///
/// * `domain_key` - Key of the domain for which to fetch changes.
/// * `unlink` - Whether to unlink changes for the requested domain.
///              Defaults to `true`.
pub fn get_changes(from: Time, apprc: &Apprc) -> Res<Vec<Changes>> {
    let mut con = db::con(&apprc.sql).unwrap();
    let rows = con
        .query(
            "
            SELECT * FROM user_change
            JOIN domain_to_user_change ON domain_to_user_change.user_change_id = user_change.id
            JOIN domain ON domain.id = domain_to_user_change.domain_id
            WHERE domain.key = $1",
            &[&domain_key],
        )
        .unwrap();
    let mut user_changes: Vec<UserChange> = Vec::new();
    let mut user_change_ids: Vec<String> = Vec::new();
    for r in rows {
        let user_change = parse_row(&r).unwrap();
        user_change_ids.push(user_change.id.to_string());
        user_changes.push(user_change);
    }

    let unlink = match unlink {
        Some(v) => v,
        None => true,
    };

    if unlink && !user_change_ids.is_empty() {
        // user changes without links are saved in the db for the complete
        // history
        let sql = format!(
            "
            DELETE FROM domain_to_user_change
            WHERE
                domain_id IN (SELECT id FROM domain WHERE key = $1)
                AND {}",
            &sql::build_where_in(
                "domain_to_user_change.user_change_id",
                &user_change_ids
            )
            .unwrap()
        );
        con.execute(&sql, &[&domain_key]).unwrap();
    }

    Ok(user_changes)
}

pub fn create(data: &CreateUserChange, apprc: &Apprc) -> Res<UserChange> {
    if data.username.is_none() && data.user_id.is_none() {
        return err::reserr("val_err", "specify either username or user_id");
    }

    let mut con = db::con(&apprc.sql).unwrap();
    let row = if data.username.is_some() {
        con.query_one(
            "
                INSERT INTO user_change (user_detached_id, action)
                VALUES ((SELECT id FROM appuser WHERE username = $1), $2)
                RETURNING *",
            &[&data.username, &data.action],
        )
        .unwrap()
    } else {
        asrt!(data.user_id.is_some());
        con.query_one(
            "
                INSERT INTO user_change (user_detached_id, action)
                VALUES ($1, $2)
                RETURNING *",
            &[&data.user_id, &data.action],
        )
        .unwrap()
    };

    let user_change = parse_row(&row).unwrap();
    add_for_all_domains(&mut con, &user_change).unwrap();
    Ok(user_change)
}

fn add_for_all_domains(con: &mut Client, user_change: &UserChange) -> Res<()> {
    let domain_ids = match con.query("SELECT id from domain", &[]) {
        Err(e) => Err(db::convert_psql_err(e)),
        Ok(rows) => Ok(rows),
    }?;
    let domain_ids_len = domain_ids.len();
    let domain_ids = domain_ids.iter().map(|r| r.get::<_, Id>("id"));

    let mut values_sql = String::new();
    for (i, domain_id) in domain_ids.enumerate() {
        if i == domain_ids_len - 1 {
            values_sql
                .push_str(&format!("({}, {})", user_change.id, domain_id));
            continue;
        }
        values_sql
            .push_str(&format!("({}, {}),\n", user_change.id, domain_id));
    }
    let sql = &format!(
        "
        INSERT INTO domain_to_user_change (user_change_id, domain_id)
        VALUES
        {}",
        values_sql.as_str()
    );
    con.batch_execute(&sql).unwrap();

    Ok(())
}
