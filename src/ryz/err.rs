use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    errcode: String,
    msg: String,
}

impl Error {
    pub fn new(errcode: Option<&str>, msg: Option<&str>) -> Self {
        Self {
            errcode: errcode.unwrap_or("err").into(),
            msg: msg.unwrap_or("").into(),
        }
    }
}

pub fn reserr<T>(
    errcode: Option<&str>,
    msg: Option<&str>,
) -> Result<T, Error> {
    Err(Error::new(errcode, msg))
}
