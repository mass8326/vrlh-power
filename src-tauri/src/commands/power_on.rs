use btleplug::platform::PeripheralId;
use tauri::{AppHandle, Emitter, Manager};

use crate::AppState;

#[tauri::command(async)]
pub async fn power_on(app: AppHandle, id: PeripheralId) -> crate::Result<()> {
    let state = app.state::<AppState>();
    let device = state.assert_device(&id).await?;

    let (tx, mut rx) = tokio::sync::mpsc::channel(1);
    tokio::spawn(async move { device.power_on(tx).await });
    while let Some(payload) = rx.recv().await {
        app.emit("device-update", payload)?;
    }

    Ok(())
}
