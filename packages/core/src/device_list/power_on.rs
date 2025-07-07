use async_trait::async_trait;
use btleplug::api::{Peripheral as _, WriteType};
use btleplug::platform::PeripheralId;
use futures::StreamExt;
use tokio::sync::mpsc::{channel, Receiver, Sender};

use crate::dto::{DevicePowerStatus, DeviceUpdatePayload};
use crate::util::assert_power_characteristic;

use super::DeviceList;

#[async_trait]
pub trait PowerOn {
    async fn power_on(&self, id: PeripheralId) -> crate::Result<Receiver<DeviceUpdatePayload>>;
}

#[async_trait]
impl PowerOn for DeviceList {
    async fn power_on(&self, id: PeripheralId) -> crate::Result<Receiver<DeviceUpdatePayload>> {
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
    let device = list
        .map
        .lock()
        .await
        .get(&id)
        .ok_or(crate::Error::Vrlh("Device not found"))?
        .clone();
    tx.send(DeviceUpdatePayload {
        id: id.clone(),
        addr: device.address().to_string(),
        name: None,
        power: DevicePowerStatus::PowerInitiated,
    })
    .await?;

    if !device.is_connected().await? {
        device.connect().await?;
    }
    let char = assert_power_characteristic(&device).await?;
    device
        .write(&char, [1].as_ref(), WriteType::WithResponse)
        .await?;
    device.subscribe(&char).await?;
    let mut events = device.notifications().await?;
    while let Some(event) = events.next().await {
        let power = DevicePowerStatus::from(event.value);
        let stop = match power {
            DevicePowerStatus::PoweredOn | DevicePowerStatus::Unknown(_) => true,
            _ => false,
        };
        tx.send(DeviceUpdatePayload {
            id: id.clone(),
            addr: device.address().to_string(),
            name: None,
            power: power,
        })
        .await?;
        if stop {
            break;
        }
    }

    device.unsubscribe(&char).await?;
    device.disconnect().await?;

    Ok(())
}
