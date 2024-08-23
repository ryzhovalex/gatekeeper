use std::{fs::File, io::Read};

use axum::{
    http::HeaderMap,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use diesel::prelude::Insertable;
use password::check_password;
use ryz::{
    err::{self, Error},
    path,
    query::Query,
    res::Res,
    time::Time,
};
use serde::{Deserialize, Serialize};
use token::{create_at, verify_rt};
use user::{get_by_id, get_by_rt, User};
use user_change::UserChange;

mod db;
mod password;
mod quco;
mod ryz;
mod schema;
mod sql;
mod token;
pub mod user;
pub mod user_change;

lazy_static::lazy_static! {
    static ref APPRC: Apprc = collect_apprc();
}

fn collect_apprc() -> Apprc {
    let mut file =
        File::open(path::cwd().unwrap().join("corund.cfg.yml")).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    return serde_yml::from_str(&content).unwrap();
}

#[derive(Debug, Deserialize)]
struct Apprc {
    sql: SqlCfg,
    domain: DomainCfg,
}

#[derive(Debug, Deserialize)]
struct DomainCfg {
    secret: String,
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
struct RtData {
    rt: String,
}

#[derive(Serialize, Deserialize)]
pub struct Reg {
    pub username: String,
    pub password: String,
    pub firstname: Option<String>,
    pub patronym: Option<String>,
    pub surname: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name=schema::appuser)]
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

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

async fn rpc_reg(headers: HeaderMap, Json(reg): Json<Reg>) -> Res<Json<User>> {
    verify_domain_secret_from_headers(headers)?;
    let con = &mut db::con().unwrap();
    let user = user::new(&reg, con).unwrap();
    Ok(Json(user))
}

async fn rpc_dereg(headers: HeaderMap, Json(query): Json<Query>) {
    verify_domain_secret_from_headers(headers).unwrap();
    let con = &mut db::con().unwrap();
    user::del(&query, con).unwrap();
}

/// Logins an user into the system.
///
/// All other login sessions are discarded (only 1 refresh token is possible
/// by default).
///
/// Returns refresh token.
async fn rpc_login(Json(login): Json<Login>) -> Res<String> {
    let con = &mut db::con().unwrap();
    let Ok((user, hpassword)) = user::get_by_username(&login.username, con)
    else {
        return err::res_msg(
            format!("invalid username {}", login.username.to_owned()).as_str(),
        );
    };
    if !check_password(&login.password, &hpassword) {
        return err::res_msg("incorrect password");
    }
    let rt = token::create_rt(user.id).unwrap();
    let con = &mut db::con().unwrap();
    user::set_rt_for_username(&login.username, &rt, con).unwrap();
    Ok(rt)
}

async fn rpc_logout(Json(rtdata): Json<RtData>) {
    let con = &mut db::con().unwrap();
    user::del_rt(&rtdata.rt, con).unwrap();
}

async fn rpc_current(Json(rtdata): Json<RtData>) -> Res<Json<User>> {
    let con = &mut db::con().unwrap();
    Ok(Json(get_by_rt(&rtdata.rt, con)?.0))
}

async fn rpc_access(Json(rtdata): Json<RtData>) -> Res<String> {
    let rt = rtdata.rt;
    let claims = verify_rt(&rt).unwrap();
    let con = &mut db::con().unwrap();
    let user = get_by_id(claims.user_id, con).unwrap();
    if user.rt != Some(rt) {
        return err::res_msg("no such refresh token for user");
    }
    // we don't store access tokens since they intended to be short-lived
    Ok(create_at(claims.user_id).unwrap())
}

async fn rpc_get_user_changes(
    headers: HeaderMap,
    Json(get_changes): Json<GetChanges>,
) -> Res<Json<Vec<UserChange>>> {
    verify_domain_secret_from_headers(headers)?;
    let con = &mut db::con().unwrap();
    let changes = user_change::get_many(get_changes.from, con).unwrap();
    Ok(Json(changes))
}

fn verify_domain_secret_from_headers(headers: HeaderMap) -> Res<()> {
    match headers.get("domain_secret") {
        Some(secret) => {
            if secret.to_str().unwrap() != APPRC.domain.secret {
                return err::res_msg("invalid secret");
            }
        }
        None => return err::res_msg("missing server api token"),
    }
    Ok(())
}

pub fn get_router() -> Router {
    Router::new()
        .route("/rpc/login", post(rpc_login))
        .route("/rpc/logout", post(rpc_logout))
        .route("/rpc/current", post(rpc_current))
        .route("/rpc/access", post(rpc_access))

        // domain-only
        .route("/rpc/server/reg", post(rpc_reg))
        .route("/rpc/server/dereg", post(rpc_dereg))
        .route("/rpc/server/get_user_changes", post(rpc_get_user_changes))
}
