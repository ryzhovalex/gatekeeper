use crate::res::Res;
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use serde::Serialize;
use sha2::Sha256;

#[derive(Serialize)]
struct RtClaims {
    user_id: u32,
}

// TODO: add exp
pub fn create_rt(user_id: u32) -> Res<String> {
    let secret: Hmac<Sha256> = Hmac::new_from_slice(b"weloveauth").unwrap();
    let user_sid = "tmp".to_string();
    let mut claims = RtClaims { user_id: user_id };
    Ok(claims.sign_with_key(&secret).unwrap())
}
