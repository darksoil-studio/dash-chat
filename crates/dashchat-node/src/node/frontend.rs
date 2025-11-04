use crate::profile::Profile;

use super::*;

impl Node {
    pub async fn set_profile(&self, profile: Profile) -> anyhow::Result<()> {
        let profile = Profile {
            name: profile.name,
            avatar: profile.avatar,
        };
        todo!();
        Ok(())
    }
}
