#![feature(bool_to_result)]

use named_id::*;

use dashchat_node::{mailbox::mem::MemMailbox, testing::*, *};

const TRACING_FILTER: &str =
    "dashchat=debug,p2panda_stream=info,p2panda_auth=warn,p2panda_spaces=info";

#[tokio::test(flavor = "multi_thread")]
#[ignore = "device groups are not supported yet"]
async fn device_group_solo() {
    dashchat_node::testing::setup_tracing(TRACING_FILTER, true);

    let mailbox = MemMailbox::new();
    let alice = TestNode::new(NodeConfig::default(), mailbox.client(), Some("alice")).await;
    let alicia = TestNode::new(NodeConfig::default(), mailbox.client(), Some("alicia")).await;

    println!("nodes:");
    println!("alice: {:?}", alice.device_id().short());
    println!("alicia: {:?}", alicia.device_id().short());

    #[cfg(feature = "p2p")]
    introduce_and_wait([&alice.network, &alicia.network]).await;

    println!("peers see each other");

    alice
        .add_contact(
            alicia
                .new_qr_code(ShareIntent::AddDevice, true)
                .await
                .unwrap(),
        )
        .await
        .unwrap();

    todo!("accept");
}
