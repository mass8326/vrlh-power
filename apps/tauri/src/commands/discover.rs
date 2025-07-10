use tauri::{AppHandle, Manager as _};
use vrlh_power_manager_core::{DeviceInfo, DeviceList};

use crate::{events::EventEmitter, AppState};

#[tauri::command(async)]
pub async fn discover(app: AppHandle, duration: u64) -> crate::Result<()> {
    let state = app.state::<AppState>();
    let devices = match state.get_devices() {
        // Initialize device list if not yet initialized
        None => {
            let init = DeviceList::init().await.inspect_err(|_| {
                let _ = app.emit_status("No bluetooth adapter available!".into());
            })?;
            let mut guard = state
                .devices
                .lock()
                .expect("Device list mutex must not be poisoned");
            if guard.is_some() {
                let msg = "Aborting potential duplicate scan!".into();
                return Err(crate::Error::VrlhApp(msg));
            }
            *guard = Some(init.clone());
            init
        }
        // Immediately send all statuses of currently available devices
        Some(list) => {
            for device in list
                .get_device_map()
                .lock()
                .expect("Device map mutex must not be poisoned")
                .values()
            {
                let (local, remote) = device.get_last_statuses();
                let info = DeviceInfo::from_device_statuses(device, local, remote);
                let _ = app.emit_device_update(info);
            }
            list
        }
    };

    let _ = app.emit_status("Scanning for lighthouses...".into());
    let mut rx = devices.start_scan(duration)?;
    while let Some(payload) = rx.recv().await {
        let _ = app.emit_device_update(payload);
    }

    let _ = app.emit_status("Done scanning for devices!".into());
    Ok(())
}
