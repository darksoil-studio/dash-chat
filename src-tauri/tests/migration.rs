use tauri_app_lib::{happ_bundle, migrate_app};
use tauri_plugin_holochain::{
    vec_to_locked, AppBundle, HolochainRuntime, HolochainRuntimeConfig, NetworkConfig,
};
use tempdir::TempDir;

pub fn old_happ_bundle() -> AppBundle {
    let bytes = include_bytes!("../../workdir/old-dash-chat.happ");
    AppBundle::decode(bytes).expect("Failed to decode dash-chat happ")
}

#[tokio::test(flavor = "multi_thread")]
async fn migrate_app() {
    let tmp = TempDir::new("dashchat").unwrap();

    let runtime = HolochainRuntime::launch(
        vec_to_locked(vec![]),
        HolochainRuntimeConfig {
            holochain_dir: tmp,
            network_config: NetworkConfig::default(),
            admin_port: None,
        },
    )
    .await
    .unwrap();

    let old_app_id = String::from("app1");
    let new_app_id = String::from("app2");

    runtime
        .install_app(old_app_id, old_happ_bundle(), None, None, None)
        .await
        .unwrap();

    migrate_app(&runtime, old_app_id, new_app_id, happ_bundle(), None)
        .await
        .unwrap();
}
