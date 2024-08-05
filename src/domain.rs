use postgres::{error::SqlState, Row};
use serde::{Deserialize, Serialize};

use crate::{
    db,
    rskit::{err, res::Res},
    token::{self, Expire},
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
    pub id: i32,
    pub key: String,
}

impl Default for Domain {
    fn default() -> Self {
        Self {
            id: -1,
            key: String::new(),
        }
    }
}

pub fn parse_row(row: &Row) -> Res<Domain> {
    Ok(Domain {
        id: row.get("id"),
        key: row.get("key"),
        ..Default::default()
    })
}

pub fn verify_secret(secret: &String, apprc: &Apprc) -> Res<Domain> {
    let mut con = db::con(&apprc.sql).unwrap();
    let row = con
        .query_one(
            "
                SELECT * FROM domain
                WHERE domain.secret = $1",
            &[&secret],
        )
        .unwrap();
    parse_row(&row)
}

pub fn init(apprc: &Apprc) -> Res<()> {
    let mut con = db::con(&apprc.sql).unwrap();
    // create non-existent domains
    for domain_cfg_unit in &apprc.domain.domains {
        match con.execute(
            "INSERT INTO domain (key, secret) VALUES ($1, $2)",
            &[&domain_cfg_unit.key, &domain_cfg_unit.secret],
        ) {
            Err(e) => {
                if e.code().unwrap().code() != "23505" {
                    return err::err(
                        "val_err",
                        e.as_db_error().unwrap().message(),
                    );
                }
            }
            Ok(_) => (),
        }
    }
    Ok(())
}
