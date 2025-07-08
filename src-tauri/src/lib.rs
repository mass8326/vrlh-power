mod commands;
mod error;
mod events;

use btleplug::platform::PeripheralId;
pub use error::*;
use tauri::{Builder, Manager};
use tokio::sync::Mutex;
use vrlh_power_manager_core::{Device, DeviceList};

#[derive(Default)]
pub struct AppState {
    devices: Mutex<Option<DeviceList>>,
}

impl AppState {
    async fn assert_devices(&self) -> crate::Result<DeviceList> {
        self.devices
            .lock()
            .await
            .clone()
            .ok_or(crate::Error::VrlhApp(
                "Device list accessed before initialization!".into(),
            ))
    }

    async fn assert_device(&self, id: &PeripheralId) -> crate::Result<Device> {
        self.assert_devices()
            .await?
            .get_device(id)
            .await
            .ok_or(crate::Error::VrlhApp(format!("Device '{id}' not found!")))
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            app.manage(AppState::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::discover,
            commands::power_on,
            commands::power_off,
        ])
        .run(tauri::generate_context!())
        .expect("Error occured while running application!");
}
