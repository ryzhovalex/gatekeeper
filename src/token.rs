use crate::err::ErrData;
use crate::rskit::time;
use crate::{rskit::res::Res, rskit::time::delta};
use hmac::{Hmac, Mac};
use jwt::VerifyWithKey;
use jwt::{SignWithKey, ToBase64};
use serde::Deserialize;
use serde::Serialize;
use sha2::Sha256;

#[derive(Serialize, Deserialize, Debug)]
pub struct UserTokenPayload {
    pub user_id: i32,
    pub exp: f64,
}

impl Expirable for UserTokenPayload {
    fn get_exp(&self) -> Res<f64> {
        Ok(*&self.exp)
    }
}

pub trait Expirable {
    fn get_exp(&self) -> Res<f64>;

    fn check_exp(&self) -> Res<f64> {
        let exp = *&self.get_exp().unwrap();
        if exp < time::utc() {
            return Err(ErrData::new("val_err", "expired token"));
        }
        Ok(exp)
    }
}

pub fn create_token(
    payload: &(impl ToBase64 + Expirable + Serialize),
    secret: &[u8],
) -> Res<String> {
    let encoded_secret: Hmac<Sha256> =
        Hmac::new_from_slice(secret.into()).unwrap();
    Ok(payload.sign_with_key(&encoded_secret).unwrap())
}

pub fn verify_token<T>(
    token: &String,
    secret: &[u8],
) -> Res<T> where T: ToBase64 + Expirable + for<'a> Deserialize<'a> {
    let encoded_secret: Hmac<Sha256> =
        Hmac::new_from_slice(secret).unwrap();
    let payload: T = token.verify_with_key(&encoded_secret).unwrap();
    payload.check_exp()?;
    Ok(payload)
}

pub fn create_rt(user_id: i32) -> Res<String> {
    let payload = UserTokenPayload {
        user_id: user_id,
        // 1 year for now
        exp: delta((360 * 24 * 60 * 60).into()),
    };
    create_token(&payload, b"weloveauth")
}

pub fn create_at(user_id: i32) -> Res<String> {
    let payload = UserTokenPayload {
        user_id: user_id,
        // 1 year for now, even for at
        exp: delta((360 * 24 * 60 * 60).into()),
    };
    create_token(&payload, b"helloworld")
}

pub fn verify_rt(rt: &String) -> Res<UserTokenPayload> {
    verify_token(rt, b"weloveauth")
}
