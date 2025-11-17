mod aliased;
mod short_id;

pub use aliased::*;
pub use short_id::*;

impl ShortId for p2panda_core::Hash {
    const PREFIX: &'static str = "H";
    fn short(&self) -> String {
        let mut s = self.to_hex();
        s.truncate(8);
        format!("{}|{s}", Self::PREFIX)
    }
}

impl ShortId for p2panda_core::PublicKey {
    const PREFIX: &'static str = "PK";
    fn short(&self) -> String {
        let mut s = self.to_hex();
        s.truncate(8);
        format!("{}|{s}", Self::PREFIX)
    }
}

impl ShortId for p2panda_spaces::OperationId {
    const PREFIX: &'static str = "OP";
    fn short(&self) -> String {
        let mut s = self.to_hex();
        s.truncate(8);
        format!("{}|{s}", Self::PREFIX)
    }
}

impl ShortId for p2panda_spaces::ActorId {
    const PREFIX: &'static str = "AI";
    fn short(&self) -> String {
        let mut s = self.to_hex();
        s.truncate(8);
        format!("{}|{s}", Self::PREFIX)
    }
}

impl AliasedId for p2panda_core::Hash {
    fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

impl AliasedId for p2panda_spaces::ActorId {
    fn as_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AliasedId for p2panda_core::PublicKey {
    const SHOW_SHORT_ID: bool = false;

    fn as_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AliasedId for p2panda_spaces::OperationId {
    fn as_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}
