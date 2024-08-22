use serde::{Deserialize, Serialize};

use super::res::Res;

#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    code: String,
    msg: String,
}

impl Error {
    pub fn new(code: Option<&str>, msg: Option<&str>) -> Self {
        Self {
            code: code.unwrap_or("err").into(),
            msg: msg.unwrap_or("").into(),
        }
    }
}

pub fn make_base<T>() -> Res<T> {
    Err(Error::new(None, None))
}

pub fn make_code<T>(
    code: Option<&str>
) -> Res<T> {
    Err(Error::new(code, None))
}

pub fn make_msg<T>(
    code: Option<&str>
) -> Res<T> {
    Err(Error::new(code, None))
}

pub fn make<T>(
    code: Option<&str>,
    msg: Option<&str>,
) -> Res<T> {
    Err(Error::new(code, msg))
}
