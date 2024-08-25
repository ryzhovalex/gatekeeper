use std::collections::HashMap;

use axum_test::TestServer;
use corund_lib::{
    db::{self, truncate_tables_if_allowed},
    get_router,
    user::{self, User},
    Reg,
};

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
        .post((URL.to_string() + "/server/dereg").as_str())
        .json(&HashMap::from([("username", "hello")]))
        .add_header("domain_secret", DOMAIN_SECRET)
        .await;
    assert_eq!(response.status_code(), 200);

    assert!(
        user::get_many_as_ids(con).unwrap().len() == 0,
        "must be no users"
    );
}
