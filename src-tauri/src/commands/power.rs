use btleplug::platform::PeripheralId;
use tauri::{AppHandle, Manager};
use vrlh_power_manager_core::{DeviceCommand, DeviceUpdatePayload};

use crate::{events::EventEmitter, AppState};

#[tauri::command(async)]
pub async fn power(app: AppHandle, cmd: u8, id: PeripheralId) -> crate::Result<()> {
    let command = match cmd {
        0 => DeviceCommand::Sleep,
        1 => DeviceCommand::Activate,
        2 => DeviceCommand::Standby,
        _ => return Err(crate::Error::VrlhApp("Invalid power command".into())),
    };
    handle_power_command(app, id, command).await
}

async fn handle_power_command(
    app: AppHandle,
    id: PeripheralId,
    command: DeviceCommand,
) -> crate::Result<()> {
    let state = app.state::<AppState>();
    let device = state.assert_device(&id).await?;
    let _ = app.emit_status(format!(
        r#"Sending "{command}" command to "{}""#,
        device.name()
    ));

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let clone = device.clone();
    tokio::spawn(async move { clone.power_set(tx, command).await });

    while let Some(power) = rx.recv().await {
        let payload = DeviceUpdatePayload::from_device_remote_status(&device, power);
        let _ = app.emit_device_update(payload);
    }

    Ok(())
}
