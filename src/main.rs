use std::{env, fs::File, io::Read};

use db::drop_db;
use hmac::{Hmac, Mac};
use log::{debug, error, info, warn};
use password::check_password;
use rouille::{Request, Response};
use serde::Deserialize;
use token::{create_at, verify_rt};
use user::{get_by_rt, set_rt_for_username};

#[macro_use]
extern crate rouille;
mod db;
mod err;
mod password;
mod path;
mod res;
mod time;
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
struct RtSignature {
    rt: String,
}

#[derive(Deserialize)]
struct Reg {
    pub username: String,
    pub password: String,
    pub firstname: Option<String>,
    pub patronym: Option<String>,
    pub surname: Option<String>,
}

#[allow(non_snake_case)]
fn rpc__reg(req: &&Request, apprc: &Apprc) -> Response {
    let Ok(reg) = rouille::input::json_input::<Reg>(req) else {
        let err =
            err::Err::new("val_err".to_string(), format!("invalid reg data"));
        return Response::json(&err);
    };
    let user = user::create(&reg, &apprc).unwrap();
    Response::json(&user)
}

/// Logins an user into the system.
///
/// All other login sessions are discarded (only 1 refresh token is possible
/// by default).
///
/// Returns refresh token.
#[allow(non_snake_case)]
fn rpc__login(req: &&Request, apprc: &Apprc) -> Response {
    let Ok(login) = rouille::input::json_input::<Login>(req) else {
        let err = err::Err::new(
            "val_err".to_string(),
            format!("invalid login data"),
        );
        return Response::json(&err);
    };
    let Ok((user, hpassword)) = user::get_by_username(&login.username, &apprc)
    else {
        let err = err::Err::new(
            "val_err".to_string(),
            format!("invalid username {}", login.username.to_owned()),
        );
        return Response::json(&err);
    };
    check_password(&login.password, &hpassword);
    let rt = token::create_rt(user.id).unwrap();
    user::set_rt_for_username(&login.username, &rt, &apprc).unwrap();
    Response::text(rt)
}

#[allow(non_snake_case)]
fn rpc__logout(req: &&Request, apprc: &Apprc) -> Response {
    let Ok(logout) = rouille::input::json_input::<RtSignature>(req) else {
        let err = err::Err::new(
            "val_err".to_string(),
            format!("invalid logout data"),
        );
        return Response::json(&err);
    };
    user::del_rt(&logout.rt, &apprc).unwrap();
    Response::empty_204()
}

#[allow(non_snake_case)]
fn rpc__current(req: &&Request, apprc: &Apprc) -> Response {
    let Ok(rt_signature) = rouille::input::json_input::<RtSignature>(req)
    else {
        let err = err::Err::new(
            "val_err".to_string(),
            format!("invalid rt signature"),
        );
        return Response::json(&err);
    };
    let Ok((user, _)) = get_by_rt(&rt_signature.rt, &apprc) else {
        return Response::json(&err::Err::new(
            "val_err".to_string(),
            format!("unknown refresh token"),
        ));
    };
    Response::json(&user)
}

#[allow(non_snake_case)]
fn rpc__access(req: &&Request, apprc: &Apprc) -> Response {
    let Ok(rt_signature) = rouille::input::json_input::<RtSignature>(req)
    else {
        let err = err::Err::new(
            "val_err".to_string(),
            format!("invalid rt signature"),
        );
        return Response::json(&err);
    };
    let claims = verify_rt(rt_signature.rt).unwrap();
    // we don't store access tokens since they intended to be short-lived
    Response::text(create_at(claims.user_id).unwrap())
}

#[allow(non_snake_case)]
fn rpc__get_all_user_sids(req: &&Request, apprc: &Apprc) -> Response {
    todo!();
}

#[allow(non_snake_case)]
fn rpc__get_domain_user_changes(req: &&Request, apprc: &Apprc) -> Response {
    todo!();
}

fn main() {
    colog::init();

    let mut file = File::open(path::cwd().unwrap().join("apprc.yml")).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    let apprc: Apprc = serde_yml::from_str(&content).unwrap();

    let args: Vec<String> = env::args().collect();
    if args.contains(&"-d".to_string()) {
        db::drop_db(&apprc);
        db::init(&apprc);
    }

    info!("start server");
    rouille::start_server("0.0.0.0:3000", move |request| {
        router!(request,
            (POST) (/rpc/reg) => {
                rpc__reg(&&request, &apprc)
            },
            (POST) (/rpc/login) => {
                rpc__login(&&request, &apprc)
            },
            (POST) (/rpc/logout) => {
                rpc__logout(&&request, &apprc)
            },
            (POST) (/rpc/current) => {
                rpc__current(&&request, &apprc)
            },
            (POST) (/rpc/access) => {
                rpc__access(&&request, &apprc)
            },
            (POST) (/rpc/server/domain/get_user_changes) => {
                rpc__get_domain_user_changes(&&request, &apprc)
            },
            (POST) (/rpc/server/user/get_all_sids) => {
                rpc__get_all_user_sids(&&request, &apprc)
            },
            _ => {
                rouille::Response::empty_404()
            }
        )
    });
}
