use std::time::Duration;

use p2panda_auth::Access;
use p2panda_net::ResyncConfiguration;

use crate::{
    operation::Payload,
    testing::{AliasedId, *},
    topic::Topic,
    *,
};

const TRACING_FILTER: &str =
    "dashchat=debug,p2panda_stream=info,p2panda_auth=warn,p2panda_spaces=info";

#[tokio::test(flavor = "multi_thread")]
async fn test_group_2() {
    crate::testing::setup_tracing(TRACING_FILTER);

    println!("nodes:");
    let (alice, _alice_rx) = TestNode::new(NodeConfig::default(), Some("alice")).await;
    println!("alice: {:?}", alice.public_key().short());
    let (bobbi, mut bobbi_rx) = TestNode::new(NodeConfig::default(), Some("bobbi")).await;
    println!("bobbi: {:?}", bobbi.public_key().short());

    introduce_and_wait([&alice.network, &bobbi.network]).await;

    println!("peers see each other");

    alice
        .add_friend(bobbi.new_friend_code().await.unwrap())
        .await
        .unwrap();
    bobbi
        .add_friend(alice.new_friend_code().await.unwrap())
        .await
        .unwrap();

    let chat_id = ChatId::random();
    alice.create_group(chat_id).await.unwrap();

    alice
        .add_member(chat_id, bobbi.public_key().into())
        .await
        .unwrap();

    bobbi_rx
        .watch_for(Duration::from_secs(5), |n| {
            matches!(n.payload, Payload::Inbox(InboxPayload::Friend))
        })
        .await
        .unwrap();

    bobbi_rx
        .watch_for(Duration::from_secs(5), |n| {
            matches!(
                n.payload,
                Payload::Chat(ChatPayload::JoinGroup(id)) if id == chat_id
            )
        })
        .await
        .unwrap();

    // Bobbi has joined the group via his inbox topic
    wait_for(
        Duration::from_millis(100),
        Duration::from_secs(5),
        || async {
            bobbi
                .get_groups()
                .await
                .unwrap()
                .contains(&chat_id)
                .ok_or("chat not yet found")
        },
    )
    .await
    .unwrap();

    alice.send_message(chat_id, "Hello".into()).await.unwrap();

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
    crate::testing::setup_tracing("warn,dashchat_node=warn,p2panda_stream=info,p2panda_net=error");
    // crate::testing::setup_tracing(TRACING_FILTER);

    let node_config = NodeConfig {
        resync: ResyncConfiguration::new().interval(10).poll_interval(1),
        friend_code_expiry: chrono::Duration::days(7),
    };
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
        .add_friend(bobbi.new_friend_code().await.unwrap())
        .await
        .unwrap();
    bobbi
        .add_friend(alice.new_friend_code().await.unwrap())
        .await
        .unwrap();
    bobbi
        .add_friend(carol.new_friend_code().await.unwrap())
        .await
        .unwrap();
    carol
        .add_friend(bobbi.new_friend_code().await.unwrap())
        .await
        .unwrap();

    // NOTE: not needed! "friendship" is transitive.
    // alice.add_friend(carol.me().await.unwrap()).await.unwrap();
    // carol.add_friend(alice.me().await.unwrap()).await.unwrap();

    println!("\n==> alice creates group\n");
    let chat_id = ChatId::random();
    alice.create_group(chat_id).await.unwrap();
    println!("\n==> alice adds bobbi\n");
    alice
        .add_member(chat_id, bobbi.public_key().into())
        .await
        .unwrap();

    // Bobbi has joined the group via his inbox topic and is a manager
    wait_for(
        Duration::from_millis(100),
        Duration::from_secs(10),
        || async {
            if let Ok(space) = bobbi.space(chat_id).await {
                space
                    .members()
                    .await
                    .map(|m| m.contains(&(bobbi.public_key().into(), Access::manage())))
                    .unwrap_or(false)
                    .ok_or("not a manager")
            } else {
                Err("space doesn't exist")
            }
        },
    )
    .await
    .unwrap();

    let topic: Topic = chat_id.into();
    let tt = [topic];

    println!("\n==> alice sends message\n");
    alice
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
    assert!(bobbi.op_store.is_op_processed(&topic, &bobbi_header.hash()));
    assert!(alice.op_store.is_op_processed(&topic, &bobbi_header.hash()));

    assert_eq!(
        alice.get_messages(chat_id).await.unwrap(),
        bobbi.get_messages(chat_id).await.unwrap()
    );
    assert_eq!(alice.get_messages(chat_id).await.unwrap().len(), 2);

    println!("\n==> bobbi adds carol\n");
    bobbi
        .add_member(chat_id, carol.public_key().into())
        .await
        .unwrap();

    consistency([&alice, &bobbi, &carol], &tt, &cfg)
        .await
        .unwrap();

    // Carol has joined the group via her inbox topic and is a manager
    wait_for(
        Duration::from_millis(500),
        Duration::from_secs(10),
        || async {
            if let Ok(space) = carol.space(chat_id).await {
                space
                    .members()
                    .await
                    .map(|m| m.contains(&(carol.public_key().into(), Access::manage())))
                    .unwrap_or(false)
                    .ok_or("not a manager")
            } else {
                Err("space doesn't exist")
            }
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
