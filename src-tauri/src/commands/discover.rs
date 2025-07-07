use tauri::{AppHandle, Emitter, Manager as _};
use tokio::sync::Mutex;
use vrlh_power_manager_core::CreateDiscoveryStream;

use crate::AppState;

#[tauri::command(async)]
pub async fn discover(app: AppHandle, duration: u64) -> crate::Result<()> {
    let state = app.state::<Mutex<AppState>>();
    state.lock().await.device_list.init().await?;
    let mut rx = state
        .lock()
        .await
        .device_list
        .create_discovery_stream(duration)
        .await?;
    while let Some(payload) = rx.recv().await {
        app.emit("device-update", payload)?;
    }

    Ok(())
}
