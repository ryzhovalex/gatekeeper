use core::panic;
use std::{fs::File, io::Read};

use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use postgres::Client;
use res::Res;
use rouille::{Request, Response};
use serde::{Deserialize, Serialize};
use user::set_rt_for_username;

#[macro_use]
extern crate rouille;
mod common;
mod db;
mod time;
mod err;
mod password;
mod path;
mod res;
mod token;
mod user;

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

#[derive(Deserialize)]
struct Reg {
    pub username: String,
    pub password: String,
    pub firstname: String,
    pub patronym: String,
    pub surname: String,
}

struct RpcArgs {
    req: Request,
    con: Client
}

#[allow(non_snake_case)]
fn rpc__reg(req: &&Request, apprc: &Apprc) -> Response {
    let Ok(reg) = rouille::input::json_input::<Reg>(req) else {
        let err =
            err::Err::new("val_err".to_string(), format!("invalid reg data"));
        return Response::json(&err);
    };
    let user = user::create(&reg, &apprc);
    Response::json(&user)
}

/// Logins user into the system.
///
/// All other login sessions are discarded (only 1 refresh token is possible
/// by default).
///
/// Returns refresh token.
#[allow(non_snake_case)]
fn rpc__login(req: &&Request) -> Response {
    let Ok(login) = rouille::input::json_input::<Login>(req) else {
        let err = err::Err::new(
            "val_err".to_string(),
            format!("invalid login data"),
        );
        return Response::json(&err);
    };
    let Ok(user) = user::get_by_username(&login.username) else {
        let err = err::Err::new(
            "val_err".to_string(),
            format!("invalid username {}", login.username.to_owned()),
        );
        return Response::json(&err);
    };
    let rt = token::create_rt(user.id).unwrap();
    set_rt_for_username(&login.username, &rt).unwrap();
    Response::text(rt)
}

#[allow(non_snake_case)]
fn rpc__logout(req: &&Request) -> Response {
    todo!();
}

#[allow(non_snake_case)]
fn rpc__current(req: &&Request) -> Response {
    todo!();
}

#[allow(non_snake_case)]
fn rpc__access(req: &&Request) -> Response {
    todo!();
}

#[allow(non_snake_case)]
fn rpc__get_all_user_sids(req: &&Request) -> Response {
    todo!();
}

#[allow(non_snake_case)]
fn rpc__get_domain_user_changes(req: &&Request) -> Response {
    todo!();
}

fn main() {
    colog::init();

    let mut file = File::open(path::cwd().unwrap().join("apprc.yml")).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    let apprc: Apprc = serde_yml::from_str(&content).unwrap();
    let mut con = db::con(&apprc.sql).unwrap();
    db::init(&mut con);
    con.close().unwrap();
    rouille::start_server("0.0.0.0:3000", move |request| {
        router!(request,
            (POST) (/rpc/reg) => {
                rpc__reg(&&request, &apprc)
            },
            (POST) (/rpc/login) => {
                rpc__login(&&request)
            },
            (POST) (/rpc/logout) => {
                rpc__logout(&&request)
            },
            (POST) (/rpc/current) => {
                rpc__current(&&request)
            },
            (POST) (/rpc/access) => {
                rpc__access(&&request)
            },
            (POST) (/rpc/server/domain/get_user_changes) => {
                rpc__get_domain_user_changes(&&request)
            },
            (POST) (/rpc/server/user/get_all_sids) => {
                rpc__get_all_user_sids(&&request)
            },
            _ => rouille::Response::empty_404()
        )
    });
}
