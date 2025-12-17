pub trait ShortId: std::fmt::Display {
    const PREFIX: &'static str;
    const LENGTH: usize = 4;

    fn to_short_string(&self) -> String {
        format!("{self}")
    }

    fn prefix() -> String {
        Self::PREFIX.to_string()
    }

    fn short(&self) -> String {
        let mut s = ShortId::to_short_string(self);
        s.truncate(Self::LENGTH);
        format!("{}|{s}", Self::prefix())
    }
}
