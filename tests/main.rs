//! WARN: no parallel testing is supported for now

use std::collections::HashMap;

use axum_test::TestServer;
use corund_lib::{
    db::{self, truncate_tables_if_allowed}, get_mode, get_router, quco::Query, ryz::time::utc, user::{self, User}, user_change::{self, ChangeAction, UserChange}, Reg
};
use serde_json::Value;

static URL: &str = "http://localhost:3000/rpc";
static DOMAIN_SECRET: &str = "backtomegaton";

fn new_test_server() -> TestServer {
    TestServer::new(get_router()).unwrap()
}

#[tokio::test]
async fn reg_std_ok() {
    truncate_tables_if_allowed();
    let server = new_test_server();
    let response = server
        .post((URL.to_string() + "/server/reg").as_str())
        .json(&HashMap::from([
            ("username", "hello"),
            ("password", "1234"),
        ]))
        .add_header("domain_secret", DOMAIN_SECRET)
        .await;
    assert_eq!(response.status_code(), 200);
    let user: User = response.json();

    assert_eq!(user.id, 1);
    assert_eq!(user.username, "hello");
    assert_eq!(user.firstname, None);
    assert_eq!(user.patronym, None);
    assert_eq!(user.surname, None);
    assert_eq!(user.rt, None);
}

#[tokio::test]
async fn dereg_std_ok() {
    truncate_tables_if_allowed();
    let test_start_time = utc();

    let con = &mut db::con().unwrap();
    let user = user::new(
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
        .post((URL.to_string() + "/server/dereg").as_str())
        .json(&HashMap::from([("username", "hello")]))
        .add_header("domain_secret", DOMAIN_SECRET)
        .await;
    assert_eq!(response.status_code(), 200);

    assert!(
        user::get_many_as_ids(con).unwrap().len() == 0,
        "must be no users"
    );

    let changes = user_change::get_many(test_start_time, con).unwrap();
    assert!(changes.len() == 2, "must retain new and del user changes");
    assert!(changes[0].user_id == user.id);
    assert!(changes[0].action == ChangeAction::New);
    assert!(changes[1].user_id == user.id);
    assert!(changes[1].action == ChangeAction::Del);
}

#[tokio::test]
async fn get_user_changes_std_ok() {
    truncate_tables_if_allowed();
    let test_start_time = utc();

    let con = &mut db::con().unwrap();
    let user1 = user::new(
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
    let user2 = user::new(
        &Reg {
            username: "world".to_string(),
            password: "1234".to_string(),
            firstname: None,
            patronym: None,
            surname: None,
        },
        con,
    )
    .unwrap();
    user::del(&Query::from([("username".to_string(), Value::String("world".to_string()))]), con).unwrap();

    let server = new_test_server();
    let response = server
        .post((URL.to_string() + "/server/get_user_changes").as_str())
        .json(&HashMap::from([
            ("from", test_start_time)
        ]))
        .add_header("domain_secret", DOMAIN_SECRET)
        .await;
    assert_eq!(response.status_code(), 200);
    let changes: Vec<UserChange> = response.json();
    assert!(changes.len() == 3);
    assert!(changes[0].user_id == user1.id);
    assert!(changes[0].action == ChangeAction::New);
    assert!(changes[1].user_id == user2.id);
    assert!(changes[1].action == ChangeAction::New);
    assert!(changes[2].user_id == user2.id);
    assert!(changes[2].action == ChangeAction::Del);
}
