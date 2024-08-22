use super::res::Res;

pub trait StrEnum {
    fn to_str(&self) -> &str;
    fn from_str(s: &str) -> Res<Self>
    where
        Self: Sized;
}
