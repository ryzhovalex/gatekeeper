use super::err::ErrData;

pub type Res<T> = Result<T, ErrData>;
