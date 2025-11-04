use crate::BlobHash;

pub struct Profile {
    pub name: String,
    pub avatar: Option<BlobHash>,
}
