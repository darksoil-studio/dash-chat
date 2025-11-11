#![feature(bool_to_result)]

use std::time::Duration;

use p2panda_auth::Access;
use p2panda_net::ResyncConfiguration;

use dashchat_node::{testing::*, *};

const TRACING_FILTER: &str =
    "dashchat=debug,p2panda_stream=info,p2panda_auth=warn,p2panda_spaces=info";

#[tokio::test(flavor = "multi_thread")]
async fn device_group_solo() {
    dashchat_node::testing::setup_tracing(TRACING_FILTER);

    let (alice, _alice_rx) = TestNode::new(NodeConfig::default(), Some("alice")).await;
    let (alicia, mut _alicia_rx) = TestNode::new(NodeConfig::default(), Some("alicia")).await;

    println!("nodes:");
    println!("alice: {:?}", alice.public_key().short());
    println!("alicia: {:?}", alicia.public_key().short());

    introduce_and_wait([&alice.network, &alicia.network]).await;

    println!("peers see each other");

    alice
        .add_friend(alicia.new_friend_code().await.unwrap())
        .await
        .unwrap();
    alicia
        .add_friend(alice.new_friend_code().await.unwrap())
        .await
        .unwrap();

    let chat_id = ChatId::random();
    alice.create_space(chat_id).await.unwrap();

    alice
        .add_member(chat_id, alicia.public_key().into())
        .await
        .unwrap();
}
