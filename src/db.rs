use crate::{ryz::res::Res, APPRC};
use diesel::{connection::SimpleConnection, Connection, PgConnection};

pub type Con = PgConnection;

pub type Id = i32;
#[allow(dead_code)]
pub type Sid = String;

pub fn con() -> Res<PgConnection> {
    let cfg = &APPRC.sql;
    Ok(PgConnection::establish(&cfg.url)
        .unwrap_or_else(|_| panic!("cannot connect to db")))
}

pub fn truncate_tables_if_allowed() {
    if !APPRC.sql.is_cleaning_allowed {
        return;
    }
    let con = &mut con().unwrap();
    con.batch_execute(
        "
        TRUNCATE user_change, appuser RESTART IDENTITY;
    ",
    )
    .unwrap();
}
