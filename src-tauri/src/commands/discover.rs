use futures::future::join_all;
use tauri::{AppHandle, Manager as _};
use vrlh_power_manager_core::DeviceList;

use crate::{events::EventEmitter, AppState};

#[tauri::command(async)]
pub async fn discover(app: AppHandle, duration: u64) -> crate::Result<()> {
    let state = app.state::<AppState>();
    if let Some(existing) = state.devices.lock().await.clone() {
        app.emit_status("Disconnecting from current devices...".into())?;
        let map = existing.get_device_map();
        let guard = map.lock().await;
        join_all(guard.values().map(|device| device.disconnect())).await;
    };

    let devices = DeviceList::init().await.map_err(|err| {
        let _ = app.emit_status("No bluetooth adapter available!".into());
        err
    })?;
    *state.devices.lock().await = Some(devices.clone());

    app.emit_status("Scanning for lighthouses...".into())?;
    let mut rx = devices.start_scan(duration).await?;
    while let Some(payload) = rx.recv().await {
        app.emit_device_update(payload)?;
    }

    app.emit_status("Done scanning for devices!".into())?;
    Ok(())
}
