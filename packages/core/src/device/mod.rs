use std::sync::{Arc, Mutex};

use btleplug::{
    api::{Characteristic, Peripheral as _, WriteType},
    platform::{Peripheral, PeripheralId},
};
use futures::StreamExt;
use tokio::sync::mpsc::Sender;

use crate::{
    constants::{LHV2_GATT_POWER_CHARACTERISTIC, LHV2_GATT_POWER_SERVICE},
    DevicePowerStatus,
};

#[derive(Clone, Debug)]
pub struct Device {
    peripheral: Peripheral,
    name: String,
    characteristic: Arc<Mutex<Option<Characteristic>>>,
}

pub enum PowerCommand {
    TurnOn,
    TurnOff,
}

impl From<PowerCommand> for &[u8] {
    fn from(value: PowerCommand) -> Self {
        match value {
            PowerCommand::TurnOff => &[0],
            PowerCommand::TurnOn => &[1],
        }
    }
}

impl Device {
    pub fn new(peripheral: Peripheral, name: String) -> Self {
        Self {
            peripheral,
            name,
            characteristic: Arc::new(Mutex::new(None)),
        }
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

    pub async fn power_set(
        &self,
        tx: Sender<DevicePowerStatus>,
        command: PowerCommand,
    ) -> crate::Result<()> {
        self.ensure_connected().await?;
        let characteristic = self.get_power_characteristic().await?;
        self.peripheral.subscribe(&characteristic).await?;
        let mut events = self.peripheral.notifications().await?;

        self.peripheral
            .write(&characteristic, command.into(), WriteType::WithResponse)
            .await?;
        while let Some(event) = events.next().await {
            let power = DevicePowerStatus::from(event.value);
            let stop = matches!(
                power,
                DevicePowerStatus::PoweredOn | DevicePowerStatus::PoweredOff
            );
            tx.send(power).await?;
            if stop {
                break;
            }
        }

        self.peripheral
            .unsubscribe(&characteristic)
            .await
            .map_err(|_| crate::Error::Vrlh("Could not unsubscribe from device"))
    }

    pub async fn ensure_connected(&self) -> crate::Result<()> {
        if !self.peripheral.is_connected().await? {
            self.peripheral.connect().await?;
        };
        Ok(())
    }

    pub async fn disconnect(&self) {
        self.peripheral
            .disconnect()
            .await
            .expect("Device disconnect should never error");
    }

    pub async fn get_device_status(&self) -> crate::Result<DevicePowerStatus> {
        let char = self.get_power_characteristic().await?;
        let bytes = self.peripheral.read(&char).await?;
        Ok(DevicePowerStatus::from(bytes))
    }

    pub async fn get_power_characteristic(&self) -> crate::Result<Characteristic> {
        if let Some(existing) = self
            .characteristic
            .lock()
            .map_or(None, |guard| guard.clone())
        {
            return Ok(existing);
        }
        self.peripheral.discover_services().await?;
        let found = self
            .peripheral
            .services()
            .into_iter()
            .find(|service| service.uuid == LHV2_GATT_POWER_SERVICE)
            .ok_or(crate::Error::Vrlh("Could not verify power service!"))
            .and_then(|service| {
                service
                    .characteristics
                    .into_iter()
                    .find(|char| char.uuid == LHV2_GATT_POWER_CHARACTERISTIC)
                    .ok_or(crate::Error::Vrlh("Could not verify power charateristic!"))
            })?;
        self.characteristic.clear_poison();
        *self
            .characteristic
            .lock()
            .expect("Characteristic mutex must not be poisoned") = Some(found.clone());
        Ok(found)
    }
}
