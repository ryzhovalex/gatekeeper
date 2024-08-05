use postgres::Row;
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
}

#[derive(Serialize, Deserialize)]
pub struct Domain {
    pub id: i32,
    pub key: String,
    pub pending_user_changes: Option<Vec<UserChange>>,
}

impl Default for Domain {
    fn default() -> Self {
        Self {
            id: -1,
            key: String::new(),
            pending_user_changes: None,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DomainTokenPayload {
    domain_id: i32,
    exp: f64,
}

impl Expire for DomainTokenPayload {
    fn get_exp(&self) -> Res<f64> {
        Ok(*&self.exp)
    }
}

pub fn parse_row(row: &Row) -> Res<Domain> {
    dbg!(row);
    Ok(Domain {
        id: row.get("id"),
        key: row.get("key"),
        ..Default::default()
    })
}

pub fn verify_secret(secret: &String, apprc: &Apprc) -> Res<Domain> {
    let mut con = db::con(&apprc.sql).unwrap();
    let row = con
        .query_one("SELECT * FROM domain WHERE secret = $1", &[&secret])
        .unwrap();
    parse_row(&row)
}

pub fn init(apprc: &Apprc) -> Res<()> {
    let mut con = db::con(&apprc.sql).unwrap();
    // create non-existent domains
    for domain in &apprc.domain.domains {
        let _ = con.execute("INSERT INTO domain VALUES ($1)", &[&domain.key]);
    }
    Ok(())
}
