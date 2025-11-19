//! NOTE: these tests don't test the full proper friendship flow
//! in that they don't use the inbox.

#![feature(bool_to_result)]

use std::time::Duration;

use dashchat_node::{testing::*, *};

use anyhow::anyhow;

// #[tokio::test(flavor = "multi_thread")]

// #[test]
// fn test_group_2() {
//     tokio::runtime::Builder::new_current_thread()
//         .enable_all()
//         .thread_stack_size(32_000_000)
//         .worker_threads(4)
//         .build()
//         .unwrap()
//         .block_on(async {
//             run_test_group_2().await;
//         })
// }

#[tokio::test(flavor = "multi_thread")]
async fn test_group_2() {
    // dashchat_node::testing::setup_tracing("dashchat_node=info,warn", true);
    dashchat_node::testing::setup_tracing(
        "dashchat=info,p2panda_stream=info,p2panda_auth=info,p2panda_spaces=info",
        true,
    );

    let alice = TestNode::new(NodeConfig::default(), Some("alice")).await;
    let bobbi = TestNode::new(NodeConfig::default(), Some("bobbi")).await;

    introduce_and_wait([&alice.network, &bobbi.network]).await;

    println!("nodes:");
    println!("alice: {:?}", alice.public_key().short());
    println!("bobbi: {:?}", bobbi.public_key().short());

    alice
        .behavior()
        .initiate_and_establish_contact(&bobbi, ShareIntent::AddContact)
        .await
        .unwrap();

    assert!(
        alice
            .initialized_topics()
            .await
            .contains(&alice.direct_chat_topic(bobbi.chat_actor_id()).into())
    );
    assert!(
        bobbi
            .initialized_topics()
            .await
            .contains(&bobbi.direct_chat_topic(alice.chat_actor_id()).into())
    );

    let chat_id = GroupChatId::random();
    alice.create_group_chat_space(chat_id).await.unwrap();
    alice.repair_spaces_and_publish().await.unwrap();

    alice
        .add_member(chat_id, bobbi.repped_group())
        .await
        .unwrap();

    bobbi
        .behavior()
        .accept_next_group_invitation()
        .await
        .unwrap();

    // Bobbi has joined the group via his inbox topic
    wait_for(
        Duration::from_millis(500),
        Duration::from_secs(5),
        || async {
            bobbi
                .get_groups()
                .await
                .unwrap()
                .contains(&chat_id.into())
                .ok_or("chat not yet found")
        },
    )
    .await
    .unwrap();

    alice.send_message(chat_id, "Hello".into()).await.unwrap();

    // consistency(
    //     [&alice, &bobbi],
    //     &[chat_id.into()],
    //     &ClusterConfig::default(),
    // )
    // .await
    // .unwrap();

    wait_for(
        Duration::from_millis(100),
        Duration::from_secs(5),
        || async {
            let msgs = [
                alice.get_messages(chat_id).await.unwrap().len(),
                bobbi.get_messages(chat_id).await.unwrap().len(),
            ];
            msgs.iter().all(|m| *m == 1).ok_or(msgs)
        },
    )
    .await
    .unwrap();

    let alice_messages = alice.get_messages(chat_id).await.unwrap();
    let bobbi_messages = bobbi.get_messages(chat_id).await.unwrap();

    assert_eq!(alice_messages, bobbi_messages);
    assert_eq!(
        bobbi_messages.first().map(|m| m.content.clone()),
        Some("Hello".into())
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_group_3() {
    dashchat_node::testing::setup_tracing(
        "dashchat=info,p2panda_stream=info,p2panda_auth=info,p2panda_spaces=info",
        false,
    );

    let node_config = NodeConfig::default();

    let cfg = ClusterConfig {
        poll_interval: Duration::from_millis(500),
        poll_timeout: Duration::from_secs(10),
    };
    let cluster = TestCluster::new(node_config, cfg.clone(), ["Alice", "Bobbi", "Carol"]).await;
    let [alice, bobbi, carol] = cluster.nodes().await;

    introduce_and_wait([&alice.network, &bobbi.network, &carol.network]).await;

    println!("=== NODES ===");
    println!("alice:    {:?}", alice.public_key().short());
    println!("bobbi:    {:?}", bobbi.public_key().short());
    println!("carol:    {:?}", carol.public_key().short());

    // alice -- bobbi -- carol (bobbi is the pivot)
    alice
        .behavior()
        .initiate_and_establish_contact(&bobbi, ShareIntent::AddContact)
        .await
        .unwrap();

    bobbi
        .behavior()
        .initiate_and_establish_contact(&carol, ShareIntent::AddContact)
        .await
        .unwrap();

    println!("\n==> alice creates group\n");

    let chat_id = GroupChatId::random();
    alice.create_group_chat_space(chat_id).await.unwrap();

    println!("\n==> alice adds bobbi\n");

    alice
        .add_member(chat_id, bobbi.repped_group())
        .await
        .unwrap();

    println!("\n==> bobbi accepts invitation\n");

    bobbi
        .behavior()
        .accept_next_group_invitation()
        .await
        .unwrap();

    // Bobbi has joined the group via his inbox topic and has write access
    // TODO: eventually this will be manage access
    wait_for(
        Duration::from_millis(100),
        Duration::from_secs(10),
        || async {
            let members = bobbi.space(chat_id).await?.members().await?;
            println!(
                "members: {:?}",
                members.iter().map(|(id, _)| id.alias()).collect::<Vec<_>>()
            );
            members
                .iter()
                // TODO: why is the group id not a member?
                .any(|(id, _)| id == &bobbi.public_key().into())
                .ok_or(anyhow!("not a member"))
        },
    )
    .await
    .unwrap();

    let tt = [chat_id.into()];

    println!("\n==> alice sends message\n");

    let (_, alice_header) = alice
        .send_message(chat_id, "alice is my name".into())
        .await
        .unwrap();

    consistency([&alice, &bobbi], &tt, &cfg).await.unwrap();

    assert_eq!(
        alice.get_messages(chat_id).await.unwrap(),
        bobbi.get_messages(chat_id).await.unwrap()
    );
    assert_eq!(alice.get_messages(chat_id).await.unwrap().len(), 1);

    println!("\n==> bobbi sends message\n");
    let (_, bobbi_header) = bobbi
        .send_message(chat_id, "i am bobbi".into())
        .await
        .unwrap();

    consistency([&alice, &bobbi], &tt, &cfg).await.unwrap();

    assert!(
        bobbi
            .op_store
            .is_op_processed(&chat_id.into(), &alice_header.hash())
    );
    assert!(
        alice
            .op_store
            .is_op_processed(&chat_id.into(), &bobbi_header.hash())
    );

    assert_eq!(
        alice.get_messages(chat_id).await.unwrap(),
        bobbi.get_messages(chat_id).await.unwrap()
    );
    assert_eq!(alice.get_messages(chat_id).await.unwrap().len(), 2);

    println!("\n==> bobbi adds carol\n");
    bobbi
        .add_member(chat_id, carol.repped_group())
        .await
        .unwrap();

    consistency([&alice, &bobbi, &carol], &tt, &cfg)
        .await
        .unwrap();

    // Carol has joined the group via her inbox topic and has write access
    // TODO: eventually this will be manage access
    wait_for(
        Duration::from_millis(500),
        Duration::from_secs(10),
        || async {
            carol
                .space(chat_id)
                .await?
                .members()
                .await?
                .iter()
                // TODO: why is the group id not a member?
                .any(|(id, _)| id == &carol.public_key().into())
                .ok_or(anyhow!("not a member"))
        },
    )
    .await
    .unwrap();

    println!("\n==> carol sends message\n");
    carol
        .send_message(chat_id, "watashi no namae wa carol".into())
        .await
        .unwrap();

    consistency([&alice, &bobbi, &carol], &tt, &cfg)
        .await
        .unwrap();

    wait_for(
        Duration::from_millis(500),
        Duration::from_secs(10),
        || async {
            futures::future::join_all([&alice, &bobbi, &carol].iter().map(|n| async {
                n.space(chat_id)
                    .await
                    .unwrap()
                    .members()
                    .await
                    .unwrap()
                    .len()
            }))
            .await
            .iter()
            .all(|l| *l == 3)
            .ok_or("not all members registered")
        },
    )
    .await
    .unwrap();

    wait_for(Duration::from_secs(1), Duration::from_secs(10), || async {
        let msgs = [
            alice.get_messages(chat_id).await.unwrap(),
            bobbi.get_messages(chat_id).await.unwrap(),
            carol.get_messages(chat_id).await.unwrap(),
        ];
        msgs.iter().all(|m| m.len() == 3).ok_or(msgs)
    })
    .await
    .unwrap_or_else(|e| panic!("{:#?}", e));

    let alice_messages = alice.get_messages(chat_id).await.unwrap();
    let bobbi_messages = bobbi.get_messages(chat_id).await.unwrap();
    let carol_messages = carol.get_messages(chat_id).await.unwrap();

    pretty_assertions::assert_eq!(alice_messages, bobbi_messages);
    pretty_assertions::assert_eq!(bobbi_messages, carol_messages);
}
