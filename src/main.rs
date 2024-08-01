use core::panic;
use std::{fs::File, io::Read};

use serde::{Deserialize, Serialize};

#[macro_use]
extern crate rouille;
mod db;
mod err;
mod password;
mod path;
mod res;

#[derive(Debug, Deserialize)]
struct Apprc {
    sql: SqlCfg,
}

#[derive(Debug, Deserialize)]
struct SqlCfg {
    host: String,
    port: u16,
    dbname: String,
    user: String,
    password: String,
}

impl Default for SqlCfg {
    fn default() -> SqlCfg {
        SqlCfg {
            host: "localhost".to_string(),
            port: 5432,
            dbname: "postgres".to_string(),
            user: "postgres".to_string(),
            password: "postgres".to_string(),
        }
    }
}

#[derive(Deserialize)]
struct Login {
    username: String,
    password: String,
}

fn main() {
    colog::init();

    let mut file = File::open(path::cwd().unwrap().join("apprc.yml")).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    let apprc: Apprc = serde_yml::from_str(&content).unwrap();
    let mut con = db::con(apprc.sql).unwrap();
    rouille::start_server("0.0.0.0:3000", move |request| {
        router!(request,
            (POST) (/rpc/login) => {
                let json: Login = try_or_400!(
                    rouille::input::json_input(request));
                rouille::Response::empty_204()
            },
            _ => rouille::Response::empty_404()
        )
    });
}
