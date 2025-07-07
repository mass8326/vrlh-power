use btleplug::{
    api::{Peripheral, WriteType},
    platform::PeripheralId,
};
use futures::StreamExt;
use tauri::{AppHandle, Emitter};

use crate::{
    constants::DEVICE_MAP,
    dto::{DevicePowerStatus, DeviceUpdatePayload},
    util::get_power_characteristic,
};

#[tauri::command(async)]
pub async fn power_on(app: AppHandle, id: PeripheralId) -> crate::Result<()> {
    let device_map = DEVICE_MAP.lock().await;
    let device = device_map
        .get(&id)
        .ok_or(crate::Error::Vrlh("Device not found"))?;

    app.emit(
        "device-update",
        DeviceUpdatePayload {
            id: &id,
            addr: &device.address().to_string(),
            name: None,
            power: &DevicePowerStatus::PowerInitiated,
        },
    )?;

    device.connect().await?;
    let char = get_power_characteristic(device)
        .await
        .ok_or(crate::Error::Vrlh("Device not valid"))?;
    device
        .write(&char, [1].as_ref(), WriteType::WithResponse)
        .await?;

    device.subscribe(&char).await?;
    let mut events = device.notifications().await?;
    while let Some(event) = events.next().await {
        let power = DevicePowerStatus::from(event.value);
        app.emit(
            "device-update",
            DeviceUpdatePayload {
                id: &id,
                addr: &device.address().to_string(),
                name: None,
                power: &power,
            },
        )?;
        match power {
            DevicePowerStatus::PoweredOn | DevicePowerStatus::Unknown(_) => break,
            _ => continue,
        };
    }

    device.unsubscribe(&char).await?;
    device.disconnect().await?;

    Ok(())
}
