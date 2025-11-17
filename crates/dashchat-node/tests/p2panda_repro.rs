use dashchat_node::{ChatId, DirectChatId, testing::manager::test_manager};
use p2panda_auth::Access;

#[tokio::test(flavor = "multi_thread")]
async fn test_p2panda_repro() {
    let alice = test_manager().await;
    let bobbi = test_manager().await;

    // Exchange keybundles

    alice
        .register_member(&bobbi.me().await.unwrap())
        .await
        .unwrap();
    bobbi
        .register_member(&alice.me().await.unwrap())
        .await
        .unwrap();

    // Create personal groups

    let (ga, mga, ega) = alice
        .create_group(&[(alice.id(), Access::manage())])
        .await
        .unwrap();

    let (gb, mgb, egb) = bobbi
        .create_group(&[(bobbi.id(), Access::manage())])
        .await
        .unwrap();

    for m in mga {
        bobbi.process(&m).await.unwrap();
    }

    for m in mgb {
        alice.process(&m).await.unwrap();
    }

    // Create shared space

    let (sa, msa, esa) = alice
        .create_space(
            DirectChatId::direct_chat([ga.id(), gb.id()]),
            &[(ga.id(), Access::manage()), (gb.id(), Access::manage())],
        )
        .await
        .unwrap();
}
