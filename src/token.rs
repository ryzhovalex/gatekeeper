use crate::{res::Res, time::delta};
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use serde::Serialize;
use sha2::Sha256;

#[derive(Serialize)]
struct RtClaims {
    user_id: u32,
    exp: f64,
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
