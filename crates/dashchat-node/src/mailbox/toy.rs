use super::*;

/// A client for the toy mailbox server.
#[derive(Clone)]
pub struct ToyMailboxClient {}

#[async_trait::async_trait]
impl MailboxClient for ToyMailboxClient {
    async fn publish(&self, ops: Vec<MailboxOperation>) -> Result<(), anyhow::Error> {
        todo!()
    }

    async fn fetch(
        &self,
        request: FetchRequest<MailboxOperation>,
    ) -> Result<FetchResponse<MailboxOperation>, anyhow::Error> {
        todo!()
    }
}
