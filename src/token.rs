use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use serde::Serialize;
use crate::res::Res;
use sha2::Sha256;

#[derive(Serialize)]
struct RtClaims {
    user_sid: String
}

pub fn create_rt(user_sid: String) -> Res<String> {
    let secret: Hmac<Sha256> = Hmac::new_from_slice(b"weloveauth").unwrap();
    let user_sid = "tmp".to_string();
    let mut claims = RtClaims {user_sid: user_sid};
    Ok(claims.sign_with_key(&secret).unwrap())
}
