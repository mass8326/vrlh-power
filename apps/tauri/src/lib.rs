mod commands;
mod error;
mod events;

use std::sync::Mutex;

use btleplug::platform::PeripheralId;
pub use error::*;
use futures::future::join_all;
use tauri::{
    async_runtime::block_on, generate_context, generate_handler, Builder, Manager, RunEvent,
};
use vrlh_power_manager_core::{Device, DeviceList};

#[derive(Default)]
pub struct AppState {
    devices: Mutex<Option<DeviceList>>,
}

impl AppState {
    fn get_devices(&self) -> Option<DeviceList> {
        self.devices
            .lock()
            .expect("Device list mutex must not be poisoned")
            .clone()
    }

    fn assert_devices(&self) -> crate::Result<DeviceList> {
        self.get_devices().ok_or(crate::Error::VrlhApp(
            "Device list accessed before initialization!".into(),
        ))
    }

    async fn assert_device(&self, id: &PeripheralId) -> crate::Result<Device> {
        self.assert_devices()?
            .get_device(id)
            .await
            .ok_or(crate::Error::VrlhApp(format!("Device '{id}' not found!")))
    }

    async fn disconnect_all(&self) {
        let current = self.get_devices();
        if let Some(existing) = current {
            let map = existing.get_device_map();
            let guard = map.lock().await;
            join_all(guard.values().map(Device::disconnect)).await;
        }
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
        .invoke_handler(generate_handler![commands::discover, commands::power])
        .build(generate_context!())
        .expect("Error occured while building application!")
        .run(|handle, event| {
            if let RunEvent::ExitRequested { .. } = &event {
                let state = handle.state::<AppState>();
                block_on(async move { state.disconnect_all().await });
            }
        });
}
