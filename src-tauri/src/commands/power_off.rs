use btleplug::platform::PeripheralId;
use tauri::{AppHandle, Emitter, Manager};
use vrlh_power_manager_core::PowerCommand;

use crate::{events::EventEmitter, AppState};

#[tauri::command(async)]
pub async fn power_off(app: AppHandle, id: PeripheralId) -> crate::Result<()> {
    let state = app.state::<AppState>();
    let device = state.assert_device(&id).await?;
    let _ = app.emit_status(format!(
        r#"Initiating power off for "{}""#,
        device.address()
    ));

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    tokio::spawn(async move { device.power_set(tx, PowerCommand::TurnOff).await });
    while let Some(payload) = rx.recv().await {
        let _ = app.emit("device-update", payload);
    }

    Ok(())
}
