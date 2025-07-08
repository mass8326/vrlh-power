use btleplug::platform::PeripheralId;
use tauri::{AppHandle, Manager};
use vrlh_power_manager_core::{DeviceUpdatePayload, PowerCommand};

use crate::{events::EventEmitter, AppState};

#[tauri::command(async)]
pub async fn power_off(app: AppHandle, id: PeripheralId) -> crate::Result<()> {
    handle_power_command(app, id, PowerCommand::TurnOff).await
}

#[tauri::command(async)]
pub async fn power_on(app: AppHandle, id: PeripheralId) -> crate::Result<()> {
    handle_power_command(app, id, PowerCommand::TurnOn).await
}

async fn handle_power_command(
    app: AppHandle,
    id: PeripheralId,
    command: PowerCommand,
) -> crate::Result<()> {
    let state = app.state::<AppState>();
    let device = state.assert_device(&id).await?;
    let _ = app.emit_status(format!(
        r#"Initiating power {} for "{}""#,
        match command {
            PowerCommand::TurnOff => "off",
            PowerCommand::TurnOn => "on",
        },
        device.address()
    ));

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let clone = device.clone();
    tokio::spawn(async move { clone.power_set(tx, command).await });

    while let Some(power) = rx.recv().await {
        let payload = DeviceUpdatePayload::from_device(&device, power);
        let _ = app.emit_device_update(payload);
    }

    Ok(())
}
