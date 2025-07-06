use btleplug::{
    api::{Peripheral, WriteType},
    platform::PeripheralId,
};
use tauri::{AppHandle, Emitter};

use crate::{
    constants::DEVICE_MAP,
    dto::{DevicePowerStatus, DeviceUpdatePayload},
    util::get_power_characteristic,
};

#[tauri::command(async)]
pub async fn power_off(app: AppHandle, id: PeripheralId) -> crate::Result<()> {
    app.emit(
        "device-update",
        DeviceUpdatePayload {
            id: &id,
            name: None,
            power: &DevicePowerStatus::PowerPending,
        },
    )?;
    let device_map = DEVICE_MAP.lock().await;
    let device = device_map
        .get(&id)
        .ok_or(crate::Error::Vrlh("Device not found"))?;

    device.connect().await?;
    let char = get_power_characteristic(device)
        .await
        .ok_or(crate::Error::Vrlh("Device not valid"))?;
    device
        .write(&char, [0].as_ref(), WriteType::WithResponse)
        .await?;
    device.disconnect().await?;

    app.emit(
        "device-update",
        DeviceUpdatePayload {
            id: &id,
            name: None,
            power: &DevicePowerStatus::PoweredOff,
        },
    )?;

    Ok(())
}
