use std::{env, fs::File, io::Read};

use db::Id;
use domain::{Domain, DomainCfg};
use hmac::{Hmac, Mac};
use log::{debug, error, info, warn};
use password::check_password;
use rouille::{Request, Response, ResponseBody};
use rskit::{
    err::{self, ErrData},
    path,
    query::Query,
    res::Res,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use token::{create_at, verify_rt};
use user::{get_all_ids, get_by_id, get_by_rt, set_rt_for_username};
use user_change::{get_user_changes_for_domain, UserChange};

#[macro_use]
extern crate rouille;
mod db;
mod domain;
mod password;
mod rskit;
mod token;
mod user;
mod user_change;

#[derive(Debug, Deserialize)]
struct Apprc {
    sql: SqlCfg,
    domain: DomainCfg,
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

#[derive(Deserialize)]
struct GetUserChangesForDomain {
    unlink: Option<bool>,
}

pub fn response_err(content: &ErrData) -> Response {
    let data = serde_json::to_string(content).unwrap();

    Response {
        status_code: 400,
        headers: vec![(
            "Content-Type".into(),
            "application/json; charset=utf-8".into(),
        )],
        data: ResponseBody::from_data(data),
        upgrade: None,
    }
}

#[allow(non_snake_case)]
fn rpc__reg(req: &&Request, apprc: &Apprc) -> Response {
    let Some(server_api_token) = req.header("domain_secret") else {
        return response_err(&err::ErrData::new(
            "val_err",
            "missing server api token",
        ));
    };
    let Ok(reg) = rouille::input::json_input::<Reg>(req) else {
        return response_err(&err::ErrData::new(
            "val_err".to_string(),
            format!("invalid reg data"),
        ));
    };
    let user = user::create(&reg, &apprc).unwrap();
    Response::json(&user)
}

#[allow(non_snake_case)]
fn rpc__dereg(req: &&Request, apprc: &Apprc) -> Response {
    let Ok(searchq) = rouille::input::json_input::<Query>(req) else {
        return response_err(&err::ErrData::new(
            "val_err".to_string(),
            format!("invalid searchq data"),
        ));
    };
    user::del(&searchq, &apprc).unwrap();
    Response::empty_204()
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
        let err = err::ErrData::new(
            "val_err".to_string(),
            format!("invalid login data"),
        );
        return response_err(&err);
    };
    let Ok((user, hpassword)) =
        user::get_by_username_with_hpassword(&login.username, &apprc)
    else {
        let err = err::ErrData::new(
            "val_err".to_string(),
            format!("invalid username {}", login.username.to_owned()),
        );
        return response_err(&err);
    };
    check_password(&login.password, &hpassword);
    let rt = token::create_rt(user.id).unwrap();
    user::set_rt_for_username(&login.username, &rt, &apprc).unwrap();
    Response::text(rt)
}

#[allow(non_snake_case)]
fn rpc__logout(req: &&Request, apprc: &Apprc) -> Response {
    let Ok(logout) = rouille::input::json_input::<RtSignature>(req) else {
        let err = err::ErrData::new(
            "val_err".to_string(),
            format!("invalid logout data"),
        );
        return response_err(&err);
    };
    user::del_rt(&logout.rt, &apprc).unwrap();
    Response::empty_204()
}

#[allow(non_snake_case)]
fn rpc__current(req: &&Request, apprc: &Apprc) -> Response {
    let Ok(rt_signature) = rouille::input::json_input::<RtSignature>(req)
    else {
        let err = err::ErrData::new(
            "val_err".to_string(),
            format!("invalid rt signature"),
        );
        return Response::json(&err);
    };
    let Ok((user, _)) = get_by_rt(&rt_signature.rt, &apprc) else {
        return Response::json(&err::ErrData::new(
            "val_err".to_string(),
            format!("unknown refresh token"),
        ));
    };
    Response::json(&user)
}

fn parse<T>(req: &&Request) -> Res<T>
where
    T: DeserializeOwned,
{
    match rouille::input::json_input::<T>(req) {
        Err(e) => Err(err::ErrData::new(
            "val_err".to_string(),
            format!("invalid req body"),
        )),
        Ok(v) => Ok(v),
    }
}

#[allow(non_snake_case)]
fn rpc__access(req: &&Request, apprc: &Apprc) -> Response {
    let Ok(rt_signature) = rouille::input::json_input::<RtSignature>(req)
    else {
        let err = err::ErrData::new(
            "val_err".to_string(),
            format!("invalid rt signature"),
        );
        return Response::json(&err);
    };
    let rt = rt_signature.rt;
    let claims = verify_rt(&rt).unwrap();
    let user = get_by_id(claims.user_id, &apprc).unwrap();
    if user.rt != Some(rt) {
        return Response::json(&ErrData::new(
            "val_err",
            "no such refresh token for user",
        ));
    }
    // we don't store access tokens since they intended to be short-lived
    Response::text(create_at(claims.user_id).unwrap())
}

#[allow(non_snake_case)]
fn rpc__get_all_user_ids(req: &&Request, apprc: &Apprc) -> Res<Vec<Id>> {
    get_all_ids(&apprc)
}

#[allow(non_snake_case)]
fn rpc__get_user_changes_for_domain(
    req: &&Request,
    domain: &Domain,
    apprc: &Apprc,
) -> Res<Vec<UserChange>> {
    let body = parse::<GetUserChangesForDomain>(req)?;
    get_user_changes_for_domain(&domain.key, body.unlink, apprc)
}

fn verify_domain_secret_from_header(
    req: &&Request,
    apprc: &Apprc,
) -> Res<domain::Domain> {
    let Some(secret) = req.header("domain_secret") else {
        return err::err("val_err", "can't get secret from header");
    };
    domain::verify_secret(&secret.to_string(), &apprc)
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

    domain::init(&apprc).unwrap();

    info!("start server");
    rouille::start_server("0.0.0.0:3000", move |request| {
        router!(request,
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

            // server routes, requires registered domain with server API token
            (POST) (/rpc/server/reg) => {
                match verify_domain_secret_from_header(&&request, &apprc) {
                    Err(e) => return response_err(&e),
                    Ok(_) => ()
                }
                rpc__reg(&&request, &apprc)
            },
            (POST) (/rpc/server/dereg) => {
                match verify_domain_secret_from_header(&&request, &apprc) {
                    Err(e) => return response_err(&e),
                    Ok(_) => ()
                }
                rpc__dereg(&&request, &apprc)
            },
            (POST) (/rpc/server/get_user_changes_for_domain) => {
                let domain = match verify_domain_secret_from_header(&&request, &apprc) {
                    Err(e) => return response_err(&e),
                    Ok(d) => d
                };
                match rpc__get_user_changes_for_domain(&&request, &domain, &apprc) {
                    Err(e) => response_err(&e),
                    Ok(v) => Response::json(&v)
                }
            },
            (POST) (/rpc/server/get_all_user_ids) => {
                match verify_domain_secret_from_header(&&request, &apprc) {
                    Err(e) => return response_err(&e),
                    Ok(_) => ()
                }

                match rpc__get_all_user_ids(&&request, &apprc) {
                    Err(e) => response_err(&e),
                    Ok(v) => Response::json(&v)
                }
            },
            _ => {
                rouille::Response::empty_404()
            }
        )
    });
}
