use btleplug::platform::PeripheralId;
use tauri::{AppHandle, Manager};
use vrlh_power_manager_core::{DeviceCommand, DeviceInfo, DeviceLocalStatus};

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
    let device = state.assert_device(&id)?;
    let _ = app.emit_device_update(DeviceInfo::from_device_local_status(
        &device,
        DeviceLocalStatus::Initializing,
    ));
    let _ = app.emit_status(format!(
        r#"Sending "{command}" command to "{}""#,
        device.name()
    ));

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let clone = device.clone();
    let handle = tokio::spawn(async move { clone.power_set(tx, command).await });
    while let Some(remote) = rx.recv().await {
        let payload = DeviceInfo::from_device_remote_status(&device, remote);
        let _ = app.emit_device_update(payload);
    }
    Ok(handle.await??)
}
