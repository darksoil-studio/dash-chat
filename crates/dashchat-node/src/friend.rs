use p2panda_net::ToNetwork;

#[derive(Clone, Debug)]
pub struct Friend {
    // pub member: Member,
    pub network_tx: tokio::sync::mpsc::Sender<ToNetwork>,
}
