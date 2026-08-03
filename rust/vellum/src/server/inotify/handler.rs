use crate::server::inotify::classifier::ChangeFlags;
use crate::server::state::AppState;
use serde_json::json;
use std::sync::Arc;

pub async fn process_events(flags: ChangeFlags, state: &Arc<AppState>) {
    if flags.shelf && !flags.config {
        log::info!("Filesystem change: reloading shelf files...");
        {
            let mut query = state.query.write().await;
            query.build_cache();
        }
        let _ = state.tx.send(json!({ "type": "LOGIC_UPDATE" }).to_string());
    }

    for intf_name in flags.interfaces_asset {
        log::info!("Interface '{intf_name}' asset changed.");
        let _ = state.tx.send(
            json!({
                "type": "INTERFACE_ASSET_UPDATE",
                "name": intf_name
            })
            .to_string(),
        );
    }

    if flags.config {
        handle_config_change(state).await;
    }
}

async fn handle_config_change(state: &Arc<AppState>) {
    log::info!("Filesystem change: reloading config and logic...");

    match libvellum::lua::ResolvedConfig::load() {
        Ok(new_config) => {
            let covers = new_config.covers.clone();
            let new_interfaces = new_config.interfaces.clone();
            let new_actions = new_config.actions.clone();
            let dependencies = new_config.dependencies.clone();
            let config_path = new_config.path.clone();

            {
                let mut config_guard = state.config.write().await;
                config_guard.covers.clone_from(&covers);
                config_guard.interfaces.clone_from(&new_interfaces);
                config_guard.actions.clone_from(&new_actions);
                config_guard.resolved_dependencies.clone_from(&dependencies);
            }

            {
                let mut query = state.query.write().await;
                if let Err(e) = query.reload_manifest(&config_path) {
                    log::error!("Failed to reload logic manifest: {e}");
                }
            }

            let _ = state.tx.send(
                json!({
                    "type": "CONFIG_UPDATE",
                    "config": {
                        "covers": covers
                    }
                })
                .to_string(),
            );

            let _ = state.tx.send(json!({ "type": "LOGIC_UPDATE" }).to_string());

            for (name, cfg) in &new_interfaces {
                let _ = state.tx.send(
                    json!({
                        "type": "INTERFACE_CONFIG_UPDATE",
                        "name": name,
                        "config": cfg.config
                    })
                    .to_string(),
                );
            }
        }
        Err(e) => {
            log::error!("Failed to reload config: {e:?}");
        }
    }
}
