use serde::{Deserialize, Serialize};

use super::res::Res;

#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    code: String,
    msg: String,
}

impl Default for Error {
    fn default() -> Self {
        Self {
            code: "err".to_string(),
            msg: "".to_string(),
        }
    }
}

impl Error {
    pub fn new(code: &str, msg: &str) -> Self {
        Self {
            code: code.to_string(),
            msg: msg.to_string(),
        }
    }

    pub fn new_code(code: &str) -> Self {
        Self {
            code: code.to_string(),
            ..Default::default()
        }
    }

    pub fn new_msg(msg: &str) -> Self {
        Self {
            msg: msg.to_string(),
            ..Default::default()
        }
    }
}

pub fn make_default<T>() -> Res<T> {
    Err(Error::default())
}

pub fn make_code<T>(code: &str) -> Res<T> {
    Err(Error::new_code(code))
}

pub fn make_msg<T>(msg: &str) -> Res<T> {
    Err(Error::new_msg(msg))
}

pub fn make<T>(code: &str, msg: &str) -> Res<T> {
    Err(Error::new(code, msg))
}
