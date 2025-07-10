use tauri::{AppHandle, Manager as _};
use vrlh_power_manager_core::DeviceList;

use crate::{events::EventEmitter, AppState};

#[tauri::command(async)]
pub async fn discover(app: AppHandle, duration: u64) -> crate::Result<()> {
    let state = app.state::<AppState>();
    let existing = state
        .devices
        .lock()
        .expect("Device list mutex must not be poisoned")
        .clone();

    let devices = match existing {
        None => {
            let init = DeviceList::init().await.inspect_err(|_| {
                let _ = app.emit_status("No bluetooth adapter available!".into());
            })?;
            let mut guard = state
                .devices
                .lock()
                .expect("Device list mutex must not be poisoned");
            if guard.is_some() {
                let msg = "Aborting potential duplicate discovery!".into();
                return Err(crate::Error::VrlhApp(msg));
            }
            *guard = Some(init.clone());
            init
        }
        Some(inner) => inner,
    };

    let _ = app.emit_status("Scanning for lighthouses...".into());
    let mut rx = devices.start_scan(duration)?;
    while let Some(payload) = rx.recv().await {
        let _ = app.emit_device_update(payload);
    }

    let _ = app.emit_status("Done scanning for devices!".into());
    Ok(())
}
