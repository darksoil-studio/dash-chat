use dashchat_node::{DeviceGroupId, DirectChatId, testing::manager::test_manager};
use p2panda_auth::Access;
use p2panda_spaces::traits::AuthoredMessage;
use p2panda_spaces::traits::MessageStore;

#[tokio::test(flavor = "multi_thread")]
async fn test_p2panda_repro() {
    let alice = test_manager().await;
    let bobbi = test_manager().await;

    ////////////////////////
    // Exchange keybundles
    ////////////////////////

    alice
        .register_member(&bobbi.me().await.unwrap())
        .await
        .unwrap();
    bobbi
        .register_member(&alice.me().await.unwrap())
        .await
        .unwrap();

    let alice_device_id = DeviceGroupId::random();
    let bobbi_device_id = DeviceGroupId::random();

    /////////////////////////////////
    // Create personal device spaces
    /////////////////////////////////

    let (ga, msgs_alice, _) = alice
        .create_space(alice_device_id, &[(alice.id(), Access::manage())])
        .await
        .unwrap();

    let (gb, msgs_bobbi, _) = bobbi
        .create_space(bobbi_device_id, &[(bobbi.id(), Access::manage())])
        .await
        .unwrap();

    /////////////////////////////////
    // "Sync" and process messages
    /////////////////////////////////

    for m in msgs_alice.iter() {
        // Q: is this correct? The test fails without it.
        bobbi.store.set_message(&m.id(), &m).await.unwrap();

        bobbi.process(&m).await.unwrap();
    }

    for m in msgs_bobbi.iter() {
        // Q: is this correct? The test fails without it.
        alice.store.set_message(&m.id(), &m).await.unwrap();

        alice.process(&m).await.unwrap();
    }

    ////////////////////////////////////
    // Create shared space. This panics.
    ////////////////////////////////////

    let (s, msgs_shared, _) = alice
        .create_space(
            DirectChatId::direct_chat([ga.group_id().await.unwrap(), gb.group_id().await.unwrap()]),
            &[
                (ga.group_id().await.unwrap(), Access::write()),
                (gb.group_id().await.unwrap(), Access::write()),
            ],
        )
        .await
        .unwrap();
}
