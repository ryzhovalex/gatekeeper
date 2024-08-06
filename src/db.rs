use crate::{
    rskit::{
        err::{self, ErrData},
        res::Res,
    },
    Apprc, SqlCfg,
};
use log::{info, warn};
use postgres::{Client, NoTls};

pub type Id = i32;
pub type Sid = String;

pub fn convert_psql_err(db_err: postgres::Error) -> ErrData {
    return err::ErrData::new(
        "val_err",
        db_err.as_db_error().unwrap().message(),
    );
}

pub fn con(cfg: &SqlCfg) -> Res<Client> {
    let db = Client::connect(
        format!(
            "dbname={} host={} port={} user={} password={}",
            cfg.dbname, cfg.host, cfg.port, cfg.user, cfg.password
        )
        .as_str(),
        NoTls,
    )
    .unwrap();
    Ok(db)
}

pub fn drop_db(apprc: &Apprc) {
    warn!("drop db");
    let mut con = con(&SqlCfg {
        host: apprc.sql.host.to_owned(),
        port: apprc.sql.port.to_owned(),
        dbname: "postgres".to_string(),
        user: "postgres".to_string(),
        password: "postgres".to_string(),
    })
    .unwrap();
    con.batch_execute(
        format!("DROP DATABASE IF EXISTS {} WITH (FORCE)", &apprc.sql.dbname)
            .as_str(),
    )
    .unwrap();
    con.batch_execute(
        format!(
            "CREATE DATABASE {} WITH OWNER {}",
            &apprc.sql.dbname, &apprc.sql.user
        )
        .as_str(),
    )
    .unwrap();
}

pub fn init(apprc: &Apprc) {
    info!("init db");
    let mut con = con(&apprc.sql).unwrap();
    // we use user_change.user_detached_id to allow databases to freely
    // delete users, but save del-action changes for them
    con.batch_execute(
        "
        CREATE TABLE IF NOT EXISTS appuser (
            id SERIAL PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            hpassword TEXT NOT NULL,
            firstname TEXT,
            patronym TEXT,
            surname TEXT,
            rt TEXT
        );
        CREATE TABLE IF NOT EXISTS domain (
            id SERIAL PRIMARY KEY,
            key TEXT NOT NULL UNIQUE,
            secret TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS user_change (
            id SERIAL PRIMARY KEY,
            user_detached_id SERIAL,
            action VARCHAR(256) NOT NULL
        );
        CREATE TABLE IF NOT EXISTS domain_to_user_change (
            domain_id SERIAL REFERENCES domain(id),
            user_change_id SERIAL REFERENCES user_change(id)
        );
    ",
    )
    .unwrap();
}
