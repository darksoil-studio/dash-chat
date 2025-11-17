mod aliased;
mod short_id;

pub use aliased::*;
pub use short_id::*;

impl ShortId for p2panda_core::Hash {
    const PREFIX: &'static str = "H";

    fn to_short_string(&self) -> String {
        self.to_hex()
    }
}

impl ShortId for p2panda_core::PublicKey {
    const PREFIX: &'static str = "PK";

    fn to_short_string(&self) -> String {
        self.to_hex()
    }
}

impl ShortId for p2panda_spaces::OperationId {
    const PREFIX: &'static str = "OP";

    fn to_short_string(&self) -> String {
        self.to_hex()
    }
}

impl ShortId for p2panda_spaces::ActorId {
    const PREFIX: &'static str = "AI";

    fn to_short_string(&self) -> String {
        self.to_hex()
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
    const SHOW_SHORT_ID: bool = true;

    fn as_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}
