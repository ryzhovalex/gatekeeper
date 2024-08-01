use serde::{Deserialize, Serialize};

use crate::{common::NULL, err::Err, res::Res, Reg};

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: u32,
    pub sid: String,
    pub username: String,
    pub hpassword: String,
    pub firstname: String,
    pub patronym: String,
    pub surname: String
}

#[derive(Serialize, Deserialize)]
pub struct UserChange {
    pub id: u32,
    pub user_sid: String,
    pub action: String,
    /// Domains that haven't been notified with this change yet.
    pub pending_domains: Vec<String>
}

pub fn create(data: &Reg) -> Res<User> {
    todo!()
}

pub fn get_by_username(username: &String) -> Res<User> {
    todo!()
}

pub fn set_rt_for_username(username: &String, rt: &String) -> Res<NULL> {
    todo!()
}

pub fn get_all_sids() -> Res<Vec<User>> {
    todo!()
}

pub fn get_changes() -> Res<Vec<UserChange>> {
    todo!()
}
