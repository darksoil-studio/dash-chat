pub trait ShortId {
    const PREFIX: &'static str;

    fn short(&self) -> String;
}
