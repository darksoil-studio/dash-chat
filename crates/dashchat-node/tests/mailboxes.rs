#![feature(bool_to_result)]

use std::time::Duration;

use dashchat_node::{mailbox::mem::MemMailbox, testing::*, *};

use named_id::*;

#[tokio::test(flavor = "multi_thread")]
async fn test_mailbox_late_join() {
    dashchat_node::testing::setup_tracing(
        &"
        dashchat=info,
        p2panda_stream=warn,
        p2panda_auth=warn,
        p2panda_encryption=warn,
        p2panda_spaces=warn,
        named_id=warn
        "
        .split(',')
        .map(|s| s.trim())
        .collect::<Vec<_>>()
        .join(","),
        true,
    );

    let mb = MemMailbox::new();

    // Start with no mailbox
    let alice = TestNode::new(NodeConfig::default(), Some("alice")).await;
    let bobbi = TestNode::new(NodeConfig::default(), Some("bobbi")).await;

    let chat = alice.direct_chat_topic(bobbi.agent_id());
    alice.send_message(chat, "Hello".into()).await.unwrap();

    bobbi.add_mailbox(mb.client()).await;
    alice.add_mailbox(mb.client()).await;

    wait_for(
        Duration::from_millis(100),
        Duration::from_secs(5),
        || async {
            (bobbi.get_messages(chat).await.unwrap().len() == 1).ok_or("message not received")
        },
    )
    .await
    .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_mutliple_mailboxes() {
    dashchat_node::testing::setup_tracing(
        &"
        dashchat=info,
        p2panda_stream=warn,
        p2panda_auth=warn,
        p2panda_encryption=warn,
        p2panda_spaces=warn,
        named_id=warn
        "
        .split(',')
        .map(|s| s.trim())
        .collect::<Vec<_>>()
        .join(","),
        true,
    );

    let mb1 = MemMailbox::new();
    let mb2 = MemMailbox::new();
    let alice = TestNode::new(NodeConfig::default(), Some("alice"))
        .await
        .add_mailbox(mb1.client())
        .await;
    let bobbi = TestNode::new(NodeConfig::default(), Some("bobbi"))
        .await
        .add_mailbox(mb2.client())
        .await;

    println!("nodes:");
    println!("alice: {:?}", alice.device_id().short());
    println!("bobbi: {:?}", bobbi.device_id().short());
}
