use async_trait::async_trait;
use btleplug::api::{Peripheral as _, WriteType};
use btleplug::platform::PeripheralId;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::dto::{DevicePowerStatus, DeviceUpdatePayload};
use crate::util::assert_power_characteristic;

use super::DeviceList;

#[async_trait]
pub trait PowerOff {
    async fn power_off(&self, id: PeripheralId) -> crate::Result<Receiver<DeviceUpdatePayload>>;
}

#[async_trait]
impl PowerOff for DeviceList {
    async fn power_off(&self, id: PeripheralId) -> crate::Result<Receiver<DeviceUpdatePayload>> {
        let (tx, rx) = channel(1);
        let list = self.clone();
        tokio::spawn(async move {
            if let Err(e) = handle_power_off(list, tx, id).await {
                eprintln!("{e}");
            };
        });
        Ok(rx)
    }
}

async fn handle_power_off(
    list: DeviceList,
    tx: Sender<DeviceUpdatePayload>,
    id: PeripheralId,
) -> crate::Result<()> {
    let devices = list.map.lock().await;
    let device = devices
        .get(&id)
        .ok_or_else(|| crate::Error::Vrlh("Device not found"))?;

    tx.send(DeviceUpdatePayload {
        id: id.clone(),
        addr: device.address().to_string(),
        name: None,
        power: DevicePowerStatus::PowerPending,
    })
    .await?;

    let char = assert_power_characteristic(&device).await?;
    if !device.is_connected().await? {
        device.connect().await?;
    }
    device
        .write(&char, [0].as_ref(), WriteType::WithResponse)
        .await?;
    device.disconnect().await?;

    tx.send(DeviceUpdatePayload {
        id,
        addr: device.address().to_string(),
        name: None,
        power: DevicePowerStatus::PoweredOff,
    })
    .await?;

    Ok(())
}
