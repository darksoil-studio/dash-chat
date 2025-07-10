use anyhow::anyhow;
use holochain_client::{AdminWebsocket, AppInfo, CellInfo};
use std::time::Duration;
use tauri_plugin_holochain::*;

pub async fn with_retries<T>(
    condition: impl AsyncFn() -> anyhow::Result<T>,
    retries: usize,
) -> anyhow::Result<T> {
    let mut retry_count = 0;
    loop {
        let response = condition().await;

        match response {
            Ok(r) => {
                return Ok(r);
            }
            Err(err) => {
                log::warn!("Condition not met yet: {err:?} Retrying in 1s.");
                std::thread::sleep(Duration::from_secs(1));

                retry_count += 1;
                if retry_count == retries {
                    return Err(anyhow!("Timeout. Last error: {err:?}"));
                }
            }
        }
    }
}

pub async fn migrate_app(
    admin_ws: &AdminWebsocket,
    existing_app_id: InstalledAppId,
    new_app_id: InstalledAppId,
    new_app_bundle: AppBundle,
    new_roles_settings: Option<RoleSettingsMap>,
) -> anyhow::Result<AppInfo> {
    let apps = admin_ws.list_apps(None).await?;

    let Some(existing_app_info) = apps
        .into_iter()
        .find(|app| app.installed_app_id.eq(&existing_app_id))
    else {
        return Err(anyhow!("Existing app {} not found.", existing_app_id));
    };

    let mut new_roles_settings = new_roles_settings.unwrap_or_default();

    let mut roles_settings = RoleSettingsMap::new();

    // For every new role
    // - Check if there was an existing provisioned cell
    //   - If there wasn't, use given roles settings
    //   - If there was:
    //     - Compute new dna and compare with existing
    //       - If the dna has not changed, add the RolesSettings::UseExisting
    //       - If the dna has changed, use given roles settings
    for new_role in new_app_bundle.manifest().app_roles() {
        let new_role_settings = new_roles_settings.remove(&new_role.name);

        if let Some(new_role_settings) = &new_role_settings {
            if let RoleSettings::UseExisting { cell_id } = new_role_settings {
                roles_settings.insert(
                    new_role.name,
                    RoleSettings::UseExisting {
                        cell_id: cell_id.clone(),
                    },
                );
                continue;
            }
        };

        let existing_cells = existing_app_info.cell_info.get(&new_role.name);

        let Some(existing_cell) =
            existing_cells
                .cloned()
                .unwrap_or_default()
                .iter()
                .find_map(|c| match c {
                    CellInfo::Provisioned(c) => Some(c.clone()),
                    _ => None,
                })
        else {
            if let Some(role_settings) = new_role_settings {
                roles_settings.insert(new_role.name, role_settings);
            }
            continue;
        };

        let new_modifiers = match &new_role_settings {
            Some(RoleSettings::Provisioned { modifiers, .. }) => match modifiers {
                Some(modifiers) => match modifiers.properties.clone() {
                    Some(properties) => {
                        let bytes = SerializedBytes::try_from(properties)?;
                        Some(DnaModifiersOpt {
                            network_seed: modifiers.network_seed.clone(),
                            properties: Some(bytes),
                        })
                    }
                    None => None,
                },
                None => None,
            },
            _ => None,
        };

        let Some(new_dna_hash) =
            dna_hash_for_app_bundle_role(&new_app_bundle, &new_role.name, new_modifiers).await?
        else {
            return Err(anyhow!("Invalid new dna hash."));
        };

        if new_dna_hash.eq(&existing_cell.cell_id.dna_hash()) {
            roles_settings.insert(
                new_role.name,
                RoleSettings::UseExisting {
                    cell_id: existing_cell.cell_id,
                },
            );
        } else if let Some(role_settings) = new_role_settings {
            roles_settings.insert(new_role.name, role_settings);
        };
    }

    let roles_settings = if roles_settings.is_empty() {
        None
    } else {
        Some(roles_settings)
    };

    let app_info = admin_ws
        .install_app(InstallAppPayload {
            source: AppBundleSource::Bytes(new_app_bundle.encode()?),
            agent_key: Some(existing_app_info.agent_pub_key),
            installed_app_id: Some(new_app_id),
            roles_settings,
            network_seed: None,
            ignore_genesis_failure: false,
            allow_throwaway_random_agent_key: false,
        })
        .await?;

    Ok(app_info)
}

pub async fn dna_hash_for_app_bundle_role(
    app_bundle: &AppBundle,
    role_name: &RoleName,
    dna_modifiers: Option<DnaModifiersOpt>,
) -> anyhow::Result<Option<DnaHash>> {
    let Some(role) = app_bundle
        .manifest()
        .app_roles()
        .into_iter()
        .find(|r| r.name.eq(role_name))
    else {
        return Ok(None);
    };

    let Some(DnaLocation::Bundled(path)) = role.dna.location else {
        return Ok(None);
    };

    let Some(dna_bundle_bytes) = app_bundle.bundled_resources().get(&path) else {
        return Ok(None);
    };

    let bundle = DnaBundle::decode(dna_bundle_bytes.inner())?;

    let (dna_file, _) = bundle.into_dna_file(DnaModifiersOpt::default()).await?;

    let dna_def = dna_file.dna_def().clone();

    let dna_def = if let Some(modifiers) = dna_modifiers {
        dna_def.update_modifiers(modifiers)
    } else {
        dna_def
    };

    Ok(Some(DnaHash::with_data_sync(&dna_def)))
}
