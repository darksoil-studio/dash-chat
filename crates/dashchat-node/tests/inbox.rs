#![feature(bool_to_result)]

use std::time::Duration;

use dashchat_node::{testing::*, *};

const TRACING_FILTER: &str =
    "inbox=info,dashchat=info,p2panda_stream=info,p2panda_auth=warn,p2panda_spaces=info";

#[tokio::test(flavor = "multi_thread")]
async fn test_inbox_2() {
    dashchat_node::testing::setup_tracing(TRACING_FILTER);

    let mut alice = TestNode::behavior(NodeConfig::default(), Some("alice")).await;
    let mut bobbi = TestNode::behavior(NodeConfig::default(), Some("bobbi")).await;

    println!("nodes:");
    println!("alice: {:?}", alice.public_key().short());
    println!("bobbi: {:?}", bobbi.public_key().short());

    introduce_and_wait([&alice.network, &bobbi.network]).await;

    println!("peers see each other");

    let qr = alice
        .new_qr_code(ShareIntent::AddFriend, true)
        .await
        .unwrap();
    bobbi.add_friend(qr).await.unwrap();

    alice.accept_next_friend().await.unwrap();

    assert_eq!(
        alice.get_friends().await.unwrap(),
        vec![bobbi.chat_actor_id()]
    );
    assert_eq!(
        bobbi.get_friends().await.unwrap(),
        vec![alice.chat_actor_id()]
    );

    let direct_chat_topic = alice.direct_chat_topic(bobbi.chat_actor_id());

    tracing::info!(%direct_chat_topic, ?direct_chat_topic, "direct chat id");

    wait_for(
        Duration::from_millis(100),
        Duration::from_secs(5),
        || async { alice.space(direct_chat_topic).await.map(|_| ()) },
    )
    .await
    .unwrap();

    alice
        .send_message(
            alice.direct_chat_topic(bobbi.chat_actor_id()),
            "Hello".into(),
        )
        .await
        .unwrap();

    let chat_id = ChatId::random();
    alice.create_group_chat_space(chat_id).await.unwrap();
    alice
        .add_member(chat_id, bobbi.chat_actor_id().into())
        .await
        .unwrap();

    bobbi.accept_next_group_invitation().await.unwrap();

    alice.send_message(chat_id, "Hello".into()).await.unwrap();
}
