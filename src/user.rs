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

pub fn create(data: &Reg) -> Res<User> {
    todo!()
}

pub fn get_by_username(username: &String) -> Res<User> {
    todo!()
}

pub fn set_rt_for_username(username: &String, rt: &String) -> Res<NULL> {
    todo!()
}
