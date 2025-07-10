use btleplug::platform::PeripheralId;
use tauri::{AppHandle, Manager};
use vrlh_power_manager_core::{DeviceCommand, DeviceLocalStatus};

use crate::{events::EventEmitter, traits::EmitAppDeviceEvent, AppState};

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
    let _ = DeviceLocalStatus::Initializing.emit(&app, &device).await;
    let _ = app.emit_status(format!(
        r#"Sending "{command}" command to "{}""#,
        device.name()
    ));

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    let device_clone = device.clone();
    let command_clone = command.clone();
    let handle = tokio::spawn(async move { device_clone.power_set(tx, &command_clone).await });

    while let Some(remote) = rx.recv().await {
        let _ = remote.emit(&app, &device).await;
    }

    handle.await??;
    let _ = app.emit_status(format!(r#"Finished "{command}" for "{}""#, device.name()));
    Ok(())
}
