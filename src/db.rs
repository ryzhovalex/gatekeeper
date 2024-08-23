use crate::{
    ryz::{
        err::{self, Error},
        res::Res,
    },
    Apprc, SqlCfg, APPRC,
};
use diesel::{Connection, PgConnection};

pub type Con = PgConnection;

pub type Id = i32;
#[allow(dead_code)]
pub type Sid = String;

pub fn con() -> Res<PgConnection> {
    let cfg = &APPRC.sql;
    Ok(PgConnection::establish(&cfg.url)
        .unwrap_or_else(|_| panic!("cannot connect to db")))
}
