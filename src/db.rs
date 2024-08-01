use crate::{res::Res, SqlCfg};
use postgres::{Client, NoTls};

pub fn con(cfg: SqlCfg) -> Res<Client> {
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

pub fn init(con: &mut Client) {
    con.batch_execute(
        "
        CREATE TABLE user (
            id SERIAL PRIMARY KEY,
            sid VARCHAR(36) NOT NULL,
            username TEXT NOT NULL,
            hpassword TEXT NOT NULL
        );
        CREATE TABLE domain (
            id SERIAL PRIMARY KEY,
            key TEXT NOT NULL
        );
        CREATE TABLE user_change (
            id SERIAL PRIMARY KEY,
            user_id SERIAL REFERENCES user(id),
            domain_id SERIAL REFERENCES domain(id),
            action VARCHAR(256) NOT NULL
        );
    ",
    )
    .unwrap();
}
