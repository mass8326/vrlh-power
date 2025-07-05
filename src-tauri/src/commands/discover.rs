use std::time::Duration;

use btleplug::api::{Central, CentralEvent, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Peripheral, PeripheralId};
use futures::StreamExt;
use tauri::{AppHandle, Emitter, Manager as _};
use tokio::time::sleep;

use crate::constants::DEVICE_MAP;
use crate::dto::{DevicePowerStatus, DeviceUpdatePayload};
use crate::util::{get_default_adapter, get_power_characteristic};

#[tauri::command(async)]
pub async fn discover(app: AppHandle) -> crate::Result<()> {
    let adapter = get_default_adapter().await?;
    let events = adapter.events().await?;
    adapter.start_scan(ScanFilter::default()).await?;
    let mut limited = events.take_until(Box::pin(sleep(Duration::from_secs(10))));

    DEVICE_MAP.lock().await.clear();
    while let Some(evt) = limited.next().await {
        let CentralEvent::DeviceDiscovered(id) = evt else {
            continue;
        };
        let mut device_map = DEVICE_MAP.lock().await;
        if !device_map.contains_key(&id) {
            let device = adapter.peripheral(&id).await?;
            device_map.insert(id.clone(), device.clone());
            tokio::spawn(process_device(
                app.app_handle().clone(),
                adapter.clone(),
                id.clone(),
            ));
        }
    }
    let _ = adapter.stop_scan().await;

    Ok(())
}

async fn process_device(app: AppHandle, adapter: Adapter, id: PeripheralId) -> crate::Result<()> {
    let device = adapter.peripheral(&id).await?;
    let Some(name) = device
        .properties()
        .await?
        .and_then(|props| props.local_name)
        .and_then(|name| match name.starts_with("LHB-") {
            true => Some(name),
            false => None,
        })
    else {
        return Ok(());
    };

    if !device.is_connected().await? {
        device.connect().await?;
    }
    app.emit(
        "device-update",
        DeviceUpdatePayload {
            id: &id,
            name: Some(&name),
            power: &DevicePowerStatus::Loading,
        },
    )?;
    app.emit(
        "device-update",
        DeviceUpdatePayload {
            id: &id,
            name: Some(&name),
            power: &get_device_status(&device).await?,
        },
    )?;
    device.disconnect().await?;

    Ok(())
}

async fn get_device_status(device: &Peripheral) -> crate::Result<DevicePowerStatus> {
    let Some(char) = get_power_characteristic(device).await else {
        return Ok(DevicePowerStatus::Error(
            "Failed to verify power service!".into(),
        ));
    };
    let status = device
        .read(&char)
        .await
        .ok()
        .map(|bytes| DevicePowerStatus::from(bytes))
        .unwrap_or(DevicePowerStatus::Error(
            "Failed to verify power service!".into(),
        ));
    Ok(status)
}
