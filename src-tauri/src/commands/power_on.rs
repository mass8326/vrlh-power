use btleplug::platform::PeripheralId;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;
use vrlh_power_manager_core::PowerOn;

use crate::AppState;

#[tauri::command(async)]
pub async fn power_on(app: AppHandle, id: PeripheralId) -> crate::Result<()> {
    let state = app.state::<Mutex<AppState>>();
    let devices = &state.lock().await.device_list;
    let mut rx = devices.power_on(id).await?;
    while let Some(payload) = rx.recv().await {
        app.emit("device-update", payload)?;
    }
    Ok(())
}
