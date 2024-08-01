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
            username TEXT NOT NULL,
            hpassword TEXT NOT NULL
        )
    ",
    )
    .unwrap();
}
