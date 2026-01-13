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
    let mut config = NodeConfig::testing();
    config.mailboxes_config.success_interval = Duration::from_millis(1000);
    config.mailboxes_config.error_interval = Duration::from_millis(1000);

    // Start with no mailbox
    let alice = TestNode::new(config.clone(), Some("alice")).await;
    let bobbi = TestNode::new(config.clone(), Some("bobbi")).await;

    let qr = alice
        .new_qr_code(ShareIntent::AddContact, true)
        .await
        .unwrap();
    bobbi.add_contact(qr).await.unwrap();

    bobbi.add_mailbox(mb.client()).await;
    alice.add_mailbox(mb.client()).await;

    alice.behavior().accept_next_contact().await.unwrap();

    // NOTE: the standard "behavior" can't work here because we're explicitly
    // testing adding the mailbox late, which means the accept_next_contact part
    // will timeout until a mailbox is added. So that's why we don't do the following
    // in this special case test:
    //
    // alice
    //     .behavior()
    //     .initiate_and_establish_contact(&bobbi, ShareIntent::AddContact)
    //     .await
    //     .unwrap();

    let chat = alice.direct_chat_topic(bobbi.agent_id());
    alice.send_message(chat, "Hello".into()).await.unwrap();

    println!("=== adding mailboxes ===");

    println!("=== added mailboxes ===");

    wait_for(
        Duration::from_millis(100),
        Duration::from_secs(5),
        || async {
            (alice.get_messages(chat).await.unwrap().len() == 1
                && bobbi.get_messages(chat).await.unwrap().len() == 1)
                .ok_or("message not received")
        },
    )
    .await
    .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "this test is only really meaningful when we have groups"]
async fn test_multiple_mailboxes_group_pivot() {
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
    let alice = TestNode::new(NodeConfig::testing(), Some("alice"))
        .await
        .add_mailbox(mb1.client())
        .await;

    let bobbi = TestNode::new(NodeConfig::testing(), Some("bobbi"))
        .await
        .add_mailbox(mb1.client())
        .await
        .add_mailbox(mb2.client())
        .await;

    let carol = TestNode::new(NodeConfig::testing(), Some("carol"))
        .await
        .add_mailbox(mb2.client())
        .await;

    alice
        .behavior()
        .initiate_and_establish_contact(&bobbi, ShareIntent::AddContact)
        .await
        .unwrap();

    carol
        .behavior()
        .initiate_and_establish_contact(&bobbi, ShareIntent::AddContact)
        .await
        .unwrap();

    todo!("this test is only really meaningful when we have groups");
}
