use postgres::Row;
use serde::{Deserialize, Serialize};

use crate::{
    db,
    rskit::{err, res::Res},
    user::UserChange,
    Apprc,
};

#[derive(Debug, Deserialize)]
pub struct DomainCfg {
    pub domains: Vec<DomainCfgUnit>,
}

#[derive(Debug, Deserialize)]
pub struct DomainCfgUnit {
    pub key: String,
    pub secret: String,
}

#[derive(Serialize, Deserialize)]
pub struct Domain {
    pub key: String,
    pub secret: String,
    pub pending_user_changes: Vec<UserChange>,
}

// pub fn parse_row(row: &Row) -> Res<Domain> {
//     Ok(Domain {
//         key: row.get("key"),
//         secret: row.get("secret"),
//         pending_user_changes: row.get
//     })
// }

// pub fn get_domain_by_secret(secret: &String, apprc: &Apprc) -> Res<Domain> {
//     let mut con = db::con(&apprc.sql).unwrap();
//     let row = con.query_one("SELECT * from domain WHERE secret = $1", &[&secret]).unwrap()
//     parse_row(&row)
// }

pub fn verify_secret(secret: &String, apprc: &Apprc) -> Res<()> {
    let mut con = db::con(&apprc.sql).unwrap();
    let row = con
        .query_one("SELECT * from domain WHERE secret = $1", &[&secret])
        .unwrap();
    let real_secret: String = row.get("secret");
    match real_secret == *secret {
        true => Ok(()),
        false => err::err("val_err", "can't verify domain's secret"),
    }
}
