use std::time::Duration;

use futures::future::join_all;
use p2panda_net::{Network, NodeAddress};

use crate::{network::Topic, testing::wait_for};

pub async fn introduce_and_wait(networks: impl IntoIterator<Item = &Network<Topic>>) {
    let networks = networks.into_iter().collect::<Vec<_>>();
    let expected_peers = networks.len() - 1;
    introduce(networks.clone()).await;
    wait_for(
        Duration::from_millis(100),
        Duration::from_secs(10),
        || async {
            let peers = join_all(
                networks
                    .iter()
                    .map(|n| async { n.known_peers().await.unwrap().len() }),
            )
            .await;
            match peers.iter().all(|p| *p == expected_peers) {
                true => Ok(()),
                false => Err(peers),
            }
        },
    )
    .await
    .unwrap();
}

pub async fn introduce(networks: impl IntoIterator<Item = &Network<Topic>>) {
    let networks = networks.into_iter().collect::<Vec<_>>();
    for m in networks.iter() {
        for n in networks.iter() {
            if m.node_id() == n.node_id() {
                continue;
            }
            let m_addr = m.endpoint().node_addr().await.unwrap();
            let n_addr = n.endpoint().node_addr().await.unwrap();

            m.add_peer(NodeAddress {
                public_key: p2panda_core::PublicKey::from_bytes(n_addr.node_id.as_bytes())
                    .expect("already validated public key"),
                direct_addresses: n_addr
                    .direct_addresses
                    .iter()
                    .map(|addr| addr.to_owned())
                    .collect(),
                relay_url: None, // n_addr.relay_url.map(to_relay_url),
            })
            .await
            .unwrap();

            n.add_peer(NodeAddress {
                public_key: p2panda_core::PublicKey::from_bytes(m_addr.node_id.as_bytes())
                    .expect("already validated public key"),
                direct_addresses: m_addr
                    .direct_addresses
                    .iter()
                    .map(|addr| addr.to_owned())
                    .collect(),
                relay_url: None, // n_addr.relay_url.map(to_relay_url),
            })
            .await
            .unwrap();
        }
    }
}
