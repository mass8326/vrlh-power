use btleplug::{
    api::{Peripheral as _, WriteType},
    platform::Peripheral,
};
use futures::StreamExt;
use tokio::sync::mpsc::Sender;

use crate::{util::assert_power_characteristic, DevicePowerStatus, DeviceUpdatePayload};

#[derive(Clone, Debug)]
pub struct Device(Peripheral);

impl Device {
    pub fn new(peripheral: Peripheral) -> Self {
        Self(peripheral)
    }

    pub async fn power_off(&self, tx: Sender<DeviceUpdatePayload>) -> crate::Result<()> {
        tx.send(DeviceUpdatePayload {
            id: self.0.id(),
            addr: self.0.address().to_string(),
            name: None,
            power: DevicePowerStatus::PowerPending,
        })
        .await?;

        let char = assert_power_characteristic(&self.0).await?;
        if !self.0.is_connected().await? {
            self.0.connect().await?;
        }
        self.0
            .write(&char, [0].as_ref(), WriteType::WithResponse)
            .await?;
        self.0.disconnect().await?;

        tx.send(DeviceUpdatePayload {
            id: self.0.id(),
            addr: self.0.address().to_string(),
            name: None,
            power: DevicePowerStatus::PoweredOff,
        })
        .await?;

        Ok(())
    }

    pub async fn power_on(&self, tx: Sender<DeviceUpdatePayload>) -> crate::Result<()> {
        tx.send(DeviceUpdatePayload {
            id: self.0.id(),
            addr: self.0.address().to_string(),
            name: None,
            power: DevicePowerStatus::PowerInitiated,
        })
        .await?;
        if !self.0.is_connected().await? {
            self.0.connect().await?;
        }
        let char = assert_power_characteristic(&self.0).await?;
        self.0
            .write(&char, [1].as_ref(), WriteType::WithResponse)
            .await?;
        self.0.subscribe(&char).await?;
        let mut events = self.0.notifications().await?;
        while let Some(event) = events.next().await {
            let power = DevicePowerStatus::from(event.value);
            let stop = match power {
                DevicePowerStatus::PoweredOn | DevicePowerStatus::Unknown(_) => true,
                _ => false,
            };
            tx.send(DeviceUpdatePayload {
                id: self.0.id(),
                addr: self.0.address().to_string(),
                name: None,
                power: power,
            })
            .await?;
            if stop {
                break;
            }
        }

        self.0.unsubscribe(&char).await?;
        self.0.disconnect().await?;

        Ok(())
    }
}
