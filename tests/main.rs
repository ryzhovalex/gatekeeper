use std::collections::HashMap;

use serde::{Deserialize, Serialize};
static URL: &str = "http://localhost:3000/rpc";

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub firstname: Option<String>,
    pub patronym: Option<String>,
    pub surname: Option<String>,
    pub rt: Option<String>,
}

#[test]
fn login_std_ok() {
    let client = reqwest::blocking::Client::new();
    let user: User = client
        .post(URL.to_string() + "/server/reg")
        .json(&HashMap::from([
            ("username", "hello"),
            ("password", "1234"),
        ]))
        .header("domain_secret", "stackunderflow")
        .send()
        .unwrap()
        .json()
        .unwrap();

    assert_eq!(user.id, 1);
    assert_eq!(user.username, "hello");
    assert_eq!(user.firstname, None);
    assert_eq!(user.patronym, None);
    assert_eq!(user.surname, None);
    assert_eq!(user.rt, None);
}
