use serde::Deserialize;

use crate::{
    ryz::{err::reserr, res::Res},
    Apprc,
};

#[derive(Debug, Deserialize)]
pub struct DomainCfg {
    secret: String,
}

pub fn verify_secret(secret: &String, apprc: &Apprc) -> Res<()> {
    match secret.as_str() == apprc.domain.secret.as_str() {
        true => Ok(()),
        false => reserr(None, None),
    }
}
