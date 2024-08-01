use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Err {
    errcode: String,
    msg: String,
}

impl Err {
    pub fn new(errcode: String, msg: String) -> Self {
        Self { errcode: errcode, msg: msg }
    }
}
