use std::collections::HashMap;

use corund_lib::{get_router, user::User};
use axum_test::TestServer;
use serde::{Deserialize, Serialize};
static URL: &str = "http://localhost:3000/rpc";

fn new_test_server() -> TestServer {
    TestServer::new(get_router()).unwrap()
}

#[tokio::test]
async fn login_std_ok() {
    let server = new_test_server();
    let response = server
        .post((URL.to_string() + "/server/reg").as_str())
        .json(&HashMap::from([
            ("username", "hello"),
            ("password", "1234"),
        ]))
        .add_header("domain_secret", "stackunderflow")
        .await;
    let user: User = response.json();

    assert_eq!(user.id, 1);
    assert_eq!(user.username, "hello");
    assert_eq!(user.firstname, None);
    assert_eq!(user.patronym, None);
    assert_eq!(user.surname, None);
    assert_eq!(user.rt, None);
}
