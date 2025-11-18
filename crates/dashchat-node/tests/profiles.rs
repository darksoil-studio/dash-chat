use std::time::Duration;

use p2panda_store::LogStore;

use dashchat_node::{testing::*, *};

const TRACING_FILTER: &str =
    "dashchat=info,p2panda_stream=info,p2panda_auth=warn,p2panda_spaces=info";

#[tokio::test(flavor = "multi_thread")]
async fn test_profiles() {
    dashchat_node::testing::setup_tracing(TRACING_FILTER, true);

    println!("nodes:");
    let alice = TestNode::new(NodeConfig::default(), Some("alice")).await;
    let bobbi = TestNode::new(NodeConfig::default(), Some("bobbi")).await;
    println!("alice: {:?}", alice.public_key().short());
    println!("bobbi: {:?}", bobbi.public_key().short());

    introduce_and_wait([&alice.network, &bobbi.network]).await;

    alice
        .add_contact(
            bobbi
                .new_qr_code(ShareIntent::AddContact, true)
                .await
                .unwrap(),
        )
        .await
        .unwrap();

    bobbi.behavior().accept_next_contact().await.unwrap();

    let profile = Profile {
        name: "Alice".to_string(),
        avatar: Some("this is a picture of alice".to_string()),
    };
    alice.set_profile(profile.clone()).await.unwrap();

    // Bob has joined the group via his inbox topic
    wait_for(
        Duration::from_millis(100),
        Duration::from_secs(5),
        || async {
            bobbi
                .op_store
                .get_log(
                    &alice.public_key(),
                    &Topic::announcements(alice.chat_actor_id()).into(),
                    None,
                )
                .await
                .map_err(|_| "failed to get log")?
                .ok_or("no log found")?
                .first()
                .filter(|(_, body)| {
                    matches!(
                        Payload::try_from_body(body.as_ref().unwrap()).unwrap(),
                        Payload::Announcements(AnnouncementsPayload::SetProfile(p)) if p == profile
                    )
                })
                .ok_or("no profile found")
                .map(|_| ())
        },
    )
    .await
    .unwrap();
}
