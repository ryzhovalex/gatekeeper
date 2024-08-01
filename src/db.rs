use crate::{res::Res, SqlCfg};
use postgres::{Client, NoTls};

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

pub fn init(con: &mut Client) {
    con.batch_execute(
        "
        CREATE TABLE user (
            id SERIAL PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            hpassword TEXT NOT NULL,
            firstname TEXT,
            patronym TEXT,
            surname TEXT
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
        CREATE TABLE domain_to_user_change (
            domain_id SERIAL REFERENCES domain(id) ON UPDATE CASCADE,
            user_change_id SERIAL REFERENCES user_change(id) ON UPDATE CASCADE
        );
    ",
    )
    .unwrap();
}
