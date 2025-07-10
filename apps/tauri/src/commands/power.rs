use btleplug::platform::PeripheralId;
use tauri::{AppHandle, Manager};
use vrlh_power_manager_core::{DeviceCommand, DeviceLocalStatus};

use crate::{
    events::{EmitEvent, StatusPayload},
    traits::EmitDeviceStatus,
    AppState,
};

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
    let device = app.state::<AppState>().assert_device(&id)?;
    let _ = app.emit_device(&device, DeviceLocalStatus::Initializing);
    let _ = app.emit_event(StatusPayload::new(format!(
        r#"Sending "{command}" command to "{}""#,
        device.name()
    )));

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let device_clone = device.clone();
    let command_clone = command.clone();
    let handle = tokio::spawn(async move { device_clone.power_set(tx, command_clone).await });

    while let Some(info) = rx.recv().await {
        let _ = app.emit_event(info);
    }

    handle.await??;
    let _ = app.emit_event(StatusPayload::new(format!(
        r#"Finished "{command}" for "{}""#,
        device.name()
    )));
    Ok(())
}
