mod commands;
mod error;

pub use error::*;
use tauri::{Builder, Manager};
use tokio::sync::Mutex;
use vrlh_power_manager_core::DeviceList;

#[derive(Default)]
pub struct AppState {
    device_list: DeviceList,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            app.manage(Mutex::new(AppState::default()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::discover,
            commands::power_on,
            commands::power_off,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
