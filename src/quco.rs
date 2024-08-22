pub trait Collection<T> {
    fn to_msg(&self) -> T;
}
