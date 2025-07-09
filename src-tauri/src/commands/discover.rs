use tauri::{AppHandle, Manager as _};
use vrlh_power_manager_core::DeviceList;

use crate::{events::EventEmitter, AppState};

#[tauri::command(async)]
pub async fn discover(app: AppHandle, duration: u64) -> crate::Result<()> {
    let state = app.state::<AppState>();
    let _ = app.emit_status("Disconnecting from current devices...".into());
    state.disconnect_all().await;

    let devices = DeviceList::init().await.inspect_err(|_| {
        let _ = app.emit_status("No bluetooth adapter available!".into());
    })?;
    *state.devices.lock().await = Some(devices.clone());

    app.emit_status("Scanning for lighthouses...".into())?;
    let mut rx = devices.start_scan(duration)?;
    while let Some(payload) = rx.recv().await {
        app.emit_device_update(payload)?;
    }

    app.emit_status("Done scanning for devices!".into())?;
    Ok(())
}
