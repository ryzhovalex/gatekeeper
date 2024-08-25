//! WARN: no parallel testing is supported for now

use std::collections::HashMap;

use axum_test::TestServer;
use corund_lib::{
    db::{self, truncate_tables_if_allowed},
    get_router, token,
    user::{self, User},
    Reg,
};

static URL: &str = "http://localhost:3000/rpc";

fn new_test_server() -> TestServer {
    TestServer::new(get_router()).unwrap()
}

#[tokio::test]
async fn login_std_ok() {
    truncate_tables_if_allowed();
    let con = &mut db::con().unwrap();
    user::new(
        &Reg {
            username: "hello".to_string(),
            password: "1234".to_string(),
            firstname: None,
            patronym: None,
            surname: None,
        },
        con,
    )
    .unwrap();

    let server = new_test_server();
    let response = server
        .post((URL.to_string() + "/login").as_str())
        .json(&HashMap::from([
            ("username", "hello"),
            ("password", "1234"),
        ]))
        .await;
    assert!(response.status_code() == 200);
    let rt = response.text();

    let user = user::get_by_id(1, con).unwrap();
    assert!(user.rt == Some(rt));
}

#[tokio::test]
async fn logout_std_ok() {
    truncate_tables_if_allowed();
    let con = &mut db::con().unwrap();
    user::new(
        &Reg {
            username: "hello".to_string(),
            password: "1234".to_string(),
            firstname: None,
            patronym: None,
            surname: None,
        },
        con,
    )
    .unwrap();

    let server = new_test_server();
    let response = server
        .post((URL.to_string() + "/login").as_str())
        .json(&HashMap::from([
            ("username", "hello"),
            ("password", "1234"),
        ]))
        .await;
    assert!(response.status_code() == 200);
    let rt = response.text();
    let response = server
        .post((URL.to_string() + "/logout").as_str())
        .json(&HashMap::from([("rt", rt)]))
        .await;
    assert!(response.status_code() == 200);

    let user = user::get_by_id(1, con).unwrap();
    assert!(user.rt == None);
}

#[tokio::test]
async fn access_std_ok() {
    truncate_tables_if_allowed();
    let con = &mut db::con().unwrap();
    user::new(
        &Reg {
            username: "hello".to_string(),
            password: "1234".to_string(),
            firstname: None,
            patronym: None,
            surname: None,
        },
        con,
    )
    .unwrap();

    let server = new_test_server();
    let response = server
        .post((URL.to_string() + "/login").as_str())
        .json(&HashMap::from([
            ("username", "hello"),
            ("password", "1234"),
        ]))
        .await;
    assert!(response.status_code() == 200);
    let rt = response.text();
    let response = server
        .post((URL.to_string() + "/access").as_str())
        .json(&HashMap::from([("rt", rt)]))
        .await;
    assert!(response.status_code() == 200);
    let at = response.text();
    let payload =
        token::verify_token::<token::UserTokenPayload>(&at, b"helloworld")
            .unwrap();
    assert!(payload.user_id == 1);
}

#[tokio::test]
async fn current_std_ok() {
    truncate_tables_if_allowed();
    let con = &mut db::con().unwrap();
    let actual_user = user::new(
        &Reg {
            username: "hello".to_string(),
            password: "1234".to_string(),
            firstname: None,
            patronym: None,
            surname: None,
        },
        con,
    )
    .unwrap();

    let server = new_test_server();
    let response = server
        .post((URL.to_string() + "/login").as_str())
        .json(&HashMap::from([
            ("username", "hello"),
            ("password", "1234"),
        ]))
        .await;
    assert!(response.status_code() == 200);
    let rt = response.text();
    let response = server
        .post((URL.to_string() + "/current").as_str())
        .json(&HashMap::from([("rt", &rt)]))
        .await;
    assert!(response.status_code() == 200);
    let user: User = response.json();
    assert_eq!(user.id, actual_user.id);
    assert_eq!(user.username, actual_user.username);
    assert_eq!(user.firstname, actual_user.firstname);
    assert_eq!(user.patronym, actual_user.patronym);
    assert_eq!(user.surname, actual_user.surname);
    assert_eq!(user.rt, Some(rt));
    assert_eq!(actual_user.rt, None)
}
