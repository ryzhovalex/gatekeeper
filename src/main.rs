use std::{env, fs::File, io::Read};

use db::Con;
use diesel::prelude::Insertable;
use log::{info};
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
use user::{get_by_id, get_by_rt};
use user_change::UserChange;

#[macro_use]
extern crate rouille;
mod db;
mod password;
mod quco;
mod ryz;
mod schema;
mod sql;
mod token;
mod user;
mod user_change;

#[derive(Debug, Deserialize)]
struct Apprc {
    sql: SqlCfg,
    domain: DomainCfg,
}

#[derive(Debug, Deserialize)]
struct DomainCfg {
    secret: String
}

#[derive(Debug, Deserialize)]
struct SqlCfg {
    url: String,
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

#[derive(Insertable)]
#[diesel(table_name=schema::user)]
pub struct InsertReg {
    pub username: String,
    pub hpassword: String,
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
        return response_err(&err::Error::new_msg("missing server api token"));
    };
    let Ok(reg) = rouille::input::json_input::<Reg>(req) else {
        return response_err(&err::Error::new_msg(
            format!("invalid reg data").as_str(),
        ));
    };
    let con = &mut db::con(&apprc.sql).unwrap();
    let user = user::new(&reg, con).unwrap();
    Response::json(&user)
}

fn rpc_dereg(req: &&Request, apprc: &Apprc) -> Response {
    let Ok(sq) = rouille::input::json_input::<Query>(req) else {
        return response_err(&err::Error::new_msg(
            format!("invalid search data").as_str(),
        ));
    };
    let con = &mut db::con(&apprc.sql).unwrap();
    user::del(&sq, con).unwrap();
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
        let err = err::Error::new_msg(format!("invalid login data").as_str());
        return response_err(&err);
    };
    let con = &mut db::con(&apprc.sql).unwrap();
    let Ok((user, hpassword)) = user::get_by_username(&login.username, con)
    else {
        let err = err::Error::new_msg(
            format!("invalid username {}", login.username.to_owned()).as_str(),
        );
        return response_err(&err);
    };
    if !check_password(&login.password, &hpassword) {
        return response_err(&Error::new_msg("incorrect password"));
    }
    let rt = token::create_rt(user.id).unwrap();
    let con = &mut db::con(&apprc.sql).unwrap();
    user::set_rt_for_username(&login.username, &rt, con).unwrap();
    Response::text(rt)
}

fn rpc_logout(req: &&Request, apprc: &Apprc) -> Response {
    let Ok(logout) = rouille::input::json_input::<RtSignature>(req) else {
        let err = err::Error::new_msg(
            format!("invalid logout data").as_str(),
        );
        return response_err(&err);
    };
    let con = &mut db::con(&apprc.sql).unwrap();
    user::del_rt(&logout.rt, con).unwrap();
    Response::empty_204()
}

fn rpc_current(req: &&Request, apprc: &Apprc) -> Response {
    let Ok(rt_signature) = rouille::input::json_input::<RtSignature>(req)
    else {
        let err = err::Error::new_msg(
            format!("invalid rt signature").as_str(),
        );
        return Response::json(&err);
    };
    let con = &mut db::con(&apprc.sql).unwrap();
    let Ok((user, _)) = get_by_rt(&rt_signature.rt, con) else {
        return Response::json(&err::Error::new_msg(
            format!("unknown refresh token").as_str(),
        ));
    };
    Response::json(&user)
}

fn parse<T>(req: &&Request) -> Res<T>
where
    T: DeserializeOwned,
{
    match rouille::input::json_input::<T>(req) {
        Err(e) => Err(err::Error::new_msg(
            format!("invalid req body").as_str(),
        )),
        Ok(v) => Ok(v),
    }
}

fn rpc_access(req: &&Request, apprc: &Apprc) -> Response {
    let Ok(rt_signature) = rouille::input::json_input::<RtSignature>(req)
    else {
        let err = err::Error::new_msg(
            format!("invalid rt signature").as_str(),
        );
        return Response::json(&err);
    };
    let rt = rt_signature.rt;
    let claims = verify_rt(&rt).unwrap();
    let con = &mut db::con(&apprc.sql).unwrap();
    let user = get_by_id(claims.user_id, con).unwrap();
    if user.rt != Some(rt) {
        return Response::json(&Error::new_msg(
            "no such refresh token for user",
        ));
    }
    // we don't store access tokens since they intended to be short-lived
    Response::text(create_at(claims.user_id).unwrap())
}

fn rpc_get_user_changes(req: &&Request, apprc: &Apprc) -> Res<Vec<UserChange>> {
    let body = parse::<GetChanges>(req)?;
    let con = &mut db::con(&apprc.sql).unwrap();
    user_change::get_many(body.from, con)
}

fn verify_domain_secret_from_header(req: &&Request, apprc: &Apprc) -> Res<()> {
    let Some(secret) = req.header("domain_secret") else {
        return err::make_msg(
            "can't get secret from header",
        );
    };
    if secret != apprc.domain.secret {
        return err::make_msg("invalid secret");
    }
    Ok(())
}

fn main() {
    colog::init();

    let mut file =
        File::open(path::cwd().unwrap().join("corund.cfg.yml")).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();

    let apprc: Apprc = serde_yml::from_str(&content).unwrap();

    // let args: Vec<String> = env::args().collect();

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
            (POST) (/rpc/server/get_user_changes) => {
                match verify_domain_secret_from_header(
                    &&request, &apprc
                ) {
                    Err(e) => return response_err(&e),
                    Ok(_) => ()
                };
                match rpc_get_user_changes(
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
