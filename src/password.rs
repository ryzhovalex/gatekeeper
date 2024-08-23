use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use rand_core::OsRng;

use crate::ryz::res::Res;

pub fn hash_password(password: &String) -> Res<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    Ok(argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string())
}

pub fn check_password(password: &String, hpassword: &String) -> bool {
    let parsed_hash = PasswordHash::new(&hpassword).unwrap();
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}
