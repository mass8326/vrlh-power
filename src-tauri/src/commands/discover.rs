use tauri::{AppHandle, Manager as _};
use vrlh_power_manager_core::DeviceList;

use crate::{events::EventEmitter, AppState};

#[tauri::command(async)]
pub async fn discover(app: AppHandle, duration: u64) -> crate::Result<()> {
    let devices = DeviceList::init().await.map_err(|err| {
        let _ = app.emit_status("No bluetooth adapter available!".into());
        err
    })?;
    let state = app.state::<AppState>();
    *state.devices.lock().await = Some(devices.clone());

    app.emit_status("Scanning for lighthouses...".into())?;
    let mut rx = devices.start_scan(duration).await?;
    while let Some(payload) = rx.recv().await {
        app.emit_device_update(payload)?;
    }

    app.emit_status("Done scanning!".into())?;
    Ok(())
}
