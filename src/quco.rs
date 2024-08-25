use crate::ryz::dict;

pub trait Collection<T> {
    fn to_msg(&self) -> T;
}
pub type Query = dict::dict<String, serde_json::Value>;
