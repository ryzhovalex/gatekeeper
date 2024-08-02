use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrData {
    errcode: String,
    msg: String,
}

impl ErrData {
    pub fn new(errcode: impl Into<String>, msg: impl Into<String>) -> Self {
        Self {
            errcode: errcode.into(),
            msg: msg.into(),
        }
    }
}

pub fn err(
    errcode: impl Into<String>,
    msg: impl Into<String>,
) -> Result<(), ErrData> {
    Err(ErrData {
        errcode: errcode.into(),
        msg: msg.into(),
    })
}
