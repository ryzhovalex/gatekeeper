//! WARN: no parallel testing is supported for now

use std::collections::HashMap;

use axum_test::TestServer;
use corund_lib::{
    db::{self, truncate_tables_if_allowed},
    get_router,
    quco::Query,
    ryz::time::utc,
    user::{self, GetUsers, User},
    user_change::{self, ChangeAction, UserChange},
    Reg,
};
use serde_json::{json, Value};

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
    user::del(
        &Query::from([(
            "username".to_string(),
            Value::String("world".to_string()),
        )]),
        con,
    )
    .unwrap();

    let server = new_test_server();
    let response = server
        .post((URL.to_string() + "/server/get_user_changes").as_str())
        .json(&HashMap::from([("from", test_start_time)]))
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

#[tokio::test]
async fn get_users_direct_id_ok() {
    truncate_tables_if_allowed();
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
    user::new(
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

    let server = new_test_server();
    let response = server
        .post((URL.to_string() + "/server/get_users").as_str())
        .add_header("domain_secret", DOMAIN_SECRET)
        .json(&GetUsers {
            sq: Query::from([("id".to_string(), json!(1))]),
        })
        .await;

    assert!(response.status_code() == 200, "{}", response.text());
    let users: Vec<User> = response.json();
    assert!(users.len() == 1);
    assert!(users[0] == user1);
}

#[tokio::test]
async fn get_users_in_id_ok() {
    truncate_tables_if_allowed();
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

    let server = new_test_server();
    let response = server
        .post((URL.to_string() + "/server/get_users").as_str())
        .add_header("domain_secret", DOMAIN_SECRET)
        .json(&json!({
            "sq": json!({
                "id": json!({
                    "$in": json!(vec![1, 2])
                })
            })
        }))
        .await;

    assert!(response.status_code() == 200, "{}", response.text());
    let users: Vec<User> = response.json();
    assert!(users.len() == 2);
    assert!(users[0] == user1);
    assert!(users[1] == user2);
}
