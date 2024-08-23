use std::collections::HashMap;

use axum_test::TestServer;
use corund_lib::{db::truncate_tables_if_allowed, get_router, user::User};

static URL: &str = "http://localhost:3000/rpc";

fn new_test_server() -> TestServer {
    TestServer::new(get_router()).unwrap()
}

#[tokio::test]
async fn login_std_ok() {
    truncate_tables_if_allowed();
    let server = new_test_server();
    let response = server
        .post((URL.to_string() + "/server/reg").as_str())
        .json(&HashMap::from([
            ("username", "hello"),
            ("password", "1234"),
        ]))
        .add_header("domain_secret", "backtomegaton")
        .await;
    dbg!(response.status_code());
    let user: User = response.json();

    assert_eq!(user.id, 1);
    assert_eq!(user.username, "hello");
    assert_eq!(user.firstname, None);
    assert_eq!(user.patronym, None);
    assert_eq!(user.surname, None);
    assert_eq!(user.rt, None);
}
