mod actions;
mod app_discovery;
mod config;
mod pairings;
mod protocol;
mod static_server;
mod ws_server;

use std::process::Command;
use tauri::Emitter;
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;

pub const DEFAULT_PORT: u16 = 41234;
pub const DEFAULT_HTTP_PORT: u16 = 8090;

#[derive(serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct AgentStatus {
    agent_name: String,
    bound_port: u16,
    lan_ip: Option<String>,
    paired_count: usize,
}

#[tauri::command]
async fn list_installed_apps() -> Result<Vec<app_discovery::InstalledApp>, String> {
    app_discovery::discover().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_profile(
    state: tauri::State<'_, ws_server::SharedState>,
) -> Result<protocol::Profile, String> {
    Ok(state.profile.read().await.clone())
}

#[tauri::command]
async fn save_profile(
    profile: protocol::Profile,
    state: tauri::State<'_, ws_server::SharedState>,
) -> Result<(), String> {
    config::save(&profile).await.map_err(|e| e.to_string())?;
    *state.profile.write().await = profile.clone();
    let _ = state.profile_tx.send(profile);
    Ok(())
}

#[tauri::command]
async fn get_agent_status(
    state: tauri::State<'_, ws_server::SharedState>,
    port: tauri::State<'_, u16>,
) -> Result<AgentStatus, String> {
    Ok(AgentStatus {
        agent_name: state.agent_name.to_string(),
        bound_port: *port.inner(),
        lan_ip: local_ip_address::local_ip().ok().map(|ip| ip.to_string()),
        paired_count: state.pairings.read().await.devices.len(),
    })
}

#[tauri::command]
async fn approve_pair(
    request_id: String,
    state: tauri::State<'_, ws_server::SharedState>,
) -> Result<(), String> {
    let mut pending = state.pending_pairs.lock().await;
    if let Some(sender) = pending.remove(&request_id) {
        sender
            .send(ws_server::PairOutcome::Approved)
            .map_err(|_| "channel closed".to_string())?;
        Ok(())
    } else {
        Err("request not found or expired".into())
    }
}

#[tauri::command]
async fn reject_pair(
    request_id: String,
    state: tauri::State<'_, ws_server::SharedState>,
) -> Result<(), String> {
    let mut pending = state.pending_pairs.lock().await;
    if let Some(sender) = pending.remove(&request_id) {
        let _ = sender.send(ws_server::PairOutcome::Rejected);
    }
    Ok(())
}

#[tauri::command]
async fn list_pairings(
    state: tauri::State<'_, ws_server::SharedState>,
) -> Result<Vec<pairings::Pairing>, String> {
    Ok(state.pairings.read().await.devices.clone())
}

#[tauri::command]
async fn revoke_pairing(
    device_id: String,
    state: tauri::State<'_, ws_server::SharedState>,
) -> Result<(), String> {
    let mut db = state.pairings.write().await;
    db.remove(&device_id);
    pairings::save(&db).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,agent_lib=debug")),
        )
        .init();

    let profile = tauri::async_runtime::block_on(config::load_or_init())
        .expect("load profile");
    let pairings_db = tauri::async_runtime::block_on(pairings::load_or_init())
        .expect("load pairings");

    let (event_tx, event_rx) = mpsc::channel::<ws_server::AgentEvent>(32);
    let state = ws_server::SharedState::new(
        profile,
        pairings_db,
        hostname(),
        event_tx,
    );
    let state_for_setup = state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .manage(DEFAULT_PORT)
        .setup(move |app| {
            let app_handle = app.handle().clone();

            // Event forwarder: ws_server -> Tauri event bus
            let app_for_events = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                let mut event_rx = event_rx;
                while let Some(event) = event_rx.recv().await {
                    match event {
                        ws_server::AgentEvent::PairRequested {
                            request_id,
                            device_id,
                            device_name,
                            peer,
                        } => {
                            let _ = app_for_events.emit(
                                "pair_requested",
                                serde_json::json!({
                                    "requestId": request_id,
                                    "deviceId": device_id,
                                    "deviceName": device_name,
                                    "peer": peer,
                                }),
                            );
                        }
                    }
                }
            });

            // WS server
            tauri::async_runtime::spawn(async move {
                match ws_server::start(state_for_setup, DEFAULT_PORT).await {
                    Ok(addr) => tracing::info!(addr = %addr, "agent ready"),
                    Err(e) => tracing::error!(error = ?e, "ws server failed"),
                }
            });

            // Embedded static HTTP server for the mobile web bundle. Skips
            // silently if no Flutter web build was bundled at compile time.
            tauri::async_runtime::spawn(async move {
                match static_server::start(DEFAULT_HTTP_PORT).await {
                    Ok(addr) => tracing::info!(addr = %addr, "mobile web served"),
                    Err(e) => tracing::warn!(error = ?e, "static server skipped"),
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_installed_apps,
            get_profile,
            save_profile,
            get_agent_status,
            approve_pair,
            reject_pair,
            list_pairings,
            revoke_pairing,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn hostname() -> String {
    Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "Stream Deck Virtual".to_string())
}
