use std::{env::var, fs::File, io::Read};

use axum::{
    extract::Request,
    http::{HeaderMap, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use diesel::prelude::Insertable;
use password::check_password;
use quco::Query;
use ryz::{
    dict::dict,
    err::{self, Error},
    path,
    res::Res,
    time::Time,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use token::{new_at, verify_rt};
use tower_http::catch_panic::CatchPanicLayer;
use tower_http::cors::{Any, CorsLayer};
use user::{get_by_id, get_by_rt, GetUsers, User};
use user_change::UserChange;

pub mod db;
mod password;
pub mod quco;
pub mod ryz;
mod schema;
pub mod token;
pub mod user;
pub mod user_change;

lazy_static::lazy_static! {
    static ref APPRC: Apprc = get_apprc();
}

fn get_apprc() -> Apprc {
    let mut file =
        File::open(path::cwd().unwrap().join("corund.cfg.yml")).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let mut mode_to_apprc: dict<String, Apprc> =
        serde_yml::from_str(&content).unwrap();
    let mode = get_mode();
    mode_to_apprc.remove(&mode).unwrap()
}

pub fn get_mode() -> String {
    return match var("CORUND_MODE") {
        Err(_) => "prod".to_string(),
        Ok(mode) => mode,
    };
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
    is_cleaning_allowed: bool,
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
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
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
    let rt = token::new_rt(user.id).unwrap();
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
    Ok(new_at(claims.user_id).unwrap())
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

async fn rpc_get_users(
    headers: HeaderMap,
    Json(inp): Json<GetUsers>,
) -> Res<Json<Vec<User>>> {
    verify_domain_secret_from_headers(headers)?;
    let con = &mut db::con().unwrap();
    let users = user::get_many(inp.sq, con).unwrap();
    Ok(Json(users))
}

fn verify_domain_secret_from_headers(headers: HeaderMap) -> Res<()> {
    match headers.get("domain_secret") {
        Some(secret) => {
            if secret.to_str().unwrap() != &APPRC.domain.secret {
                return err::res_msg("invalid secret");
            }
        }
        None => return err::res_msg("missing server api token"),
    }
    Ok(())
}

async fn err_middleware(req: Request, next: Next) -> Response {
    let res = next.run(req).await;

    // TODO: handle status codes for errs somehow without knowing response
    //       body type... don't even know how, for now

    res
}

fn panic_middleware(
    panic_err: Box<dyn std::any::Any + Send + 'static>,
) -> Response {
    let msg = if let Some(s) = panic_err.downcast_ref::<String>() {
        s.clone()
    } else if let Some(s) = panic_err.downcast_ref::<&str>() {
        s.to_string()
    } else {
        "panic".to_string()
    };
    let body = json!({
        "code": "panic",
        "msg": {
            "msg": msg
        }
    });

    (
        StatusCode::BAD_REQUEST,
        Json(body),
    )
        .into_response()
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
        .route("/rpc/server/get_users", post(rpc_get_users))
        .layer(CatchPanicLayer::custom(panic_middleware))
        .layer(middleware::from_fn(err_middleware))
        .layer(
            CorsLayer::new()
                .allow_methods([Method::POST])
                .allow_origin(Any)
                .allow_headers(Any),
        )
}
