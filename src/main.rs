use std::{env, fs::File, io::Read};

use change::{get_changes, Changes, UserChange};
use domain::DomainCfg;
use hmac::{Hmac, Mac};
use log::{debug, error, info, warn};
use password::check_password;
use rouille::{Request, Response, ResponseBody};
use ryz::{
    err::{self, Error},
    path,
    query::Query,
    res::Res,
    time::Time,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use token::{create_at, verify_rt};
use user::{get_all_ids, get_by_id, get_by_rt, set_rt_for_username};

#[macro_use]
extern crate rouille;
mod change;
mod db;
mod domain;
mod password;
mod ryz;
mod sql;
mod token;
mod user;

#[derive(Debug, Deserialize)]
struct Apprc {
    sql: SqlCfg,
    domain: DomainCfg,
}

#[derive(Debug, Deserialize)]
struct SqlCfg {
    url: String,
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
struct GetChanges {
    from: Time,
}

pub fn response_err(errdata: &Error) -> Response {
    let data = serde_json::to_string(errdata).unwrap();

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

fn rpc_reg(req: &&Request, apprc: &Apprc) -> Response {
    let Some(server_api_token) = req.header("domain_secret") else {
        return response_err(&err::Error::new(
            Some("val_err"),
            Some("missing server api token"),
        ));
    };
    let Ok(reg) = rouille::input::json_input::<Reg>(req) else {
        return response_err(&err::Error::new(
            Some("val_err"),
            Some(format!("invalid reg data").as_str()),
        ));
    };
    let user = user::create(&reg, &apprc).unwrap();
    Response::json(&user)
}

fn rpc_dereg(req: &&Request, apprc: &Apprc) -> Response {
    let Ok(searchq) = rouille::input::json_input::<Query>(req) else {
        return response_err(&err::Error::new(
            Some("val_err"),
            Some(format!("invalid search data").as_str()),
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
fn rpc_login(req: &&Request, apprc: &Apprc) -> Response {
    let Ok(login) = rouille::input::json_input::<Login>(req) else {
        let err = err::Error::new(
            Some("val_err"),
            Some(format!("invalid login data").as_str()),
        );
        return response_err(&err);
    };
    let Ok((user, hpassword)) =
        user::get_by_username_with_hpassword(&login.username, &apprc)
    else {
        let err = err::Error::new(
            Some("auth_err"),
            Some(
                format!("invalid username {}", login.username.to_owned())
                    .as_str(),
            ),
        );
        return response_err(&err);
    };
    if !check_password(&login.password, &hpassword) {
        return response_err(&Error::new(
            Some("auth_err"),
            Some("incorrect password"),
        ));
    }
    let rt = token::create_rt(user.id).unwrap();
    user::set_rt_for_username(&login.username, &rt, &apprc).unwrap();
    Response::text(rt)
}

fn rpc_logout(req: &&Request, apprc: &Apprc) -> Response {
    let Ok(logout) = rouille::input::json_input::<RtSignature>(req) else {
        let err = err::Error::new(
            Some("val_err"),
            Some(format!("invalid logout data").as_str()),
        );
        return response_err(&err);
    };
    user::del_rt(&logout.rt, &apprc).unwrap();
    Response::empty_204()
}

fn rpc_current(req: &&Request, apprc: &Apprc) -> Response {
    let Ok(rt_signature) = rouille::input::json_input::<RtSignature>(req)
    else {
        let err = err::Error::new(
            Some("val_err"),
            Some(format!("invalid rt signature").as_str()),
        );
        return Response::json(&err);
    };
    let Ok((user, _)) = get_by_rt(&rt_signature.rt, &apprc) else {
        return Response::json(&err::Error::new(
            Some("val_err"),
            Some(format!("unknown refresh token").as_str()),
        ));
    };
    Response::json(&user)
}

fn parse<T>(req: &&Request) -> Res<T>
where
    T: DeserializeOwned,
{
    match rouille::input::json_input::<T>(req) {
        Err(e) => Err(err::Error::new(
            Some("val_err"),
            Some(format!("invalid req body").as_str()),
        )),
        Ok(v) => Ok(v),
    }
}

fn rpc_access(req: &&Request, apprc: &Apprc) -> Response {
    let Ok(rt_signature) = rouille::input::json_input::<RtSignature>(req)
    else {
        let err = err::Error::new(
            Some("val_err"),
            Some(format!("invalid rt signature").as_str()),
        );
        return Response::json(&err);
    };
    let rt = rt_signature.rt;
    let claims = verify_rt(&rt).unwrap();
    let user = get_by_id(claims.user_id, &apprc).unwrap();
    if user.rt != Some(rt) {
        return Response::json(&Error::new(
            Some("val_err"),
            Some("no such refresh token for user"),
        ));
    }
    // we don't store access tokens since they intended to be short-lived
    Response::text(create_at(claims.user_id).unwrap())
}

fn rpc_get_modifications(req: &&Request, apprc: &Apprc) -> Res<Vec<Changes>> {
    let body = parse::<GetChanges>(req)?;
    get_changes(body.from, apprc)
}

fn verify_domain_secret_from_header(req: &&Request, apprc: &Apprc) -> Res<()> {
    let Some(secret) = req.header("domain_secret") else {
        return err::reserr(
            Some("val_err"),
            Some("can't get secret from header"),
        );
    };
    domain::verify_secret(&secret.to_string(), &apprc)
}

fn main() {
    colog::init();

    let mut file =
        File::open(path::cwd().unwrap().join("corund.cfg.yml")).unwrap();
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
            (POST) (/rpc/login) => {
                rpc_login(&&request, &apprc)
            },
            (POST) (/rpc/logout) => {
                rpc_logout(&&request, &apprc)
            },
            (POST) (/rpc/current) => {
                rpc_current(&&request, &apprc)
            },
            (POST) (/rpc/access) => {
                rpc_access(&&request, &apprc)
            },

            // server routes, requires registered domain with server API token
            (POST) (/rpc/server/reg) => {
                match verify_domain_secret_from_header(&&request, &apprc) {
                    Err(e) => return response_err(&e),
                    Ok(_) => ()
                }
                rpc_reg(&&request, &apprc)
            },
            (POST) (/rpc/server/dereg) => {
                match verify_domain_secret_from_header(&&request, &apprc) {
                    Err(e) => return response_err(&e),
                    Ok(_) => ()
                }
                rpc_dereg(&&request, &apprc)
            },
            (POST) (/rpc/server/get_modifications) => {
                match verify_domain_secret_from_header(
                    &&request, &apprc
                ) {
                    Err(e) => return response_err(&e),
                    Ok(_) => ()
                };
                match rpc_get_modifications(
                    &&request, &apprc
                ) {
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
