use crate::{res::Res, time::delta};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use jwt::VerifyWithKey;
use serde::Deserialize;
use serde::Serialize;
use sha2::Sha256;

#[derive(Serialize, Deserialize, Debug)]
pub struct RtClaims {
    pub user_id: u32,
    pub exp: f64,
}

pub fn create_rt(user_id: u32) -> Res<String> {
    let secret: Hmac<Sha256> = Hmac::new_from_slice(b"weloveauth").unwrap();
    let claims = RtClaims {
        user_id: user_id,
        // 1 year for now
        exp: delta((360 * 24 * 60 * 60).into()),
    };
    Ok(claims.sign_with_key(&secret).unwrap())
}

pub fn create_at(user_id: u32) -> Res<String> {
    let secret: Hmac<Sha256> = Hmac::new_from_slice(b"helloworld").unwrap();
    let claims = RtClaims {
        user_id: user_id,
        // 1 year for now, even for at
        exp: delta((360 * 24 * 60 * 60).into()),
    };
    Ok(claims.sign_with_key(&secret).unwrap())
}

pub fn verify_rt(rt: String) -> Res<RtClaims> {
    let secret: Hmac<Sha256> = Hmac::new_from_slice(b"weloveauth").unwrap();
    let claims: RtClaims = rt.verify_with_key(&secret).unwrap();
    Ok(claims)
}
