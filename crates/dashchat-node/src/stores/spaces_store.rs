use crate::chat::ChatId;
use crate::spaces::SpaceControlMessage;

// Conversion traits between ChatId and TestSpaceId
impl From<ChatId> for p2panda_spaces::test_utils::TestSpaceId {
    fn from(chat_id: ChatId) -> Self {
        // Convert the first 8 bytes of the ChatId to a usize
        let bytes = chat_id.0;
        let mut result = 0u64;
        for (i, &byte) in bytes.iter().enumerate().take(8) {
            result |= (byte as u64) << (i * 8);
        }
        result as usize
    }
}

impl From<p2panda_spaces::test_utils::TestSpaceId> for ChatId {
    fn from(test_id: p2panda_spaces::test_utils::TestSpaceId) -> Self {
        // Convert usize back to ChatId by padding with zeros
        let mut bytes = [0u8; 32];
        let id_bytes = test_id.to_le_bytes();
        bytes[..id_bytes.len()].copy_from_slice(&id_bytes);
        ChatId(bytes)
    }
}

pub type SpacesStore =
    p2panda_spaces::test_utils::store::MemoryStore<ChatId, SpaceControlMessage, ()>;
