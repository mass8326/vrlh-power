use btleplug::{
    api::{Peripheral as _, WriteType},
    platform::{Peripheral, PeripheralId},
};
use futures::StreamExt;
use tokio::sync::mpsc::Sender;

use crate::{util::assert_power_characteristic, DevicePowerStatus, DeviceUpdatePayload};

#[derive(Clone, Debug)]
pub struct Device {
    peripheral: Peripheral,
    name: String,
}

impl Device {
    pub fn new(peripheral: Peripheral, name: String) -> Self {
        Self { peripheral, name }
    }

    pub fn id(&self) -> PeripheralId {
        self.peripheral.id()
    }

    pub fn address(&self) -> String {
        self.peripheral.address().to_string()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub async fn power_off(&self, tx: Sender<DeviceUpdatePayload>) -> crate::Result<()> {
        tx.send(DeviceUpdatePayload {
            id: self.id(),
            addr: self.address(),
            name: None,
            power: DevicePowerStatus::PowerPending,
        })
        .await?;

        let char = assert_power_characteristic(&self.peripheral).await?;
        if !self.peripheral.is_connected().await? {
            self.peripheral.connect().await?;
        }
        self.peripheral
            .write(&char, [0].as_ref(), WriteType::WithResponse)
            .await?;
        self.peripheral.disconnect().await?;

        tx.send(DeviceUpdatePayload {
            id: self.id(),
            addr: self.address(),
            name: None,
            power: DevicePowerStatus::PoweredOff,
        })
        .await?;

        Ok(())
    }

    pub async fn power_on(&self, tx: Sender<DeviceUpdatePayload>) -> crate::Result<()> {
        tx.send(DeviceUpdatePayload {
            id: self.id(),
            addr: self.address(),
            name: None,
            power: DevicePowerStatus::PowerInitiated,
        })
        .await?;
        if !self.peripheral.is_connected().await? {
            self.peripheral.connect().await?;
        }
        let char = assert_power_characteristic(&self.peripheral).await?;
        self.peripheral
            .write(&char, [1].as_ref(), WriteType::WithResponse)
            .await?;
        self.peripheral.subscribe(&char).await?;
        let mut events = self.peripheral.notifications().await?;
        while let Some(event) = events.next().await {
            let power = DevicePowerStatus::from(event.value);
            let stop = match power {
                DevicePowerStatus::PoweredOn | DevicePowerStatus::Unknown(_) => true,
                _ => false,
            };
            tx.send(DeviceUpdatePayload {
                id: self.id(),
                addr: self.address(),
                name: None,
                power: power,
            })
            .await?;
            if stop {
                break;
            }
        }

        self.peripheral.unsubscribe(&char).await?;
        self.peripheral.disconnect().await?;

        Ok(())
    }
}
