use btleplug::{
    api::{Peripheral as _, WriteType},
    platform::{Peripheral, PeripheralId},
};
use futures::{try_join, StreamExt};
use tokio::sync::mpsc::Sender;

use crate::{util::assert_power_characteristic, DevicePowerStatus, DeviceUpdatePayload};

#[derive(Clone, Debug)]
pub struct Device {
    peripheral: Peripheral,
    name: String,
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

    pub async fn power_set(
        &self,
        tx: Sender<DeviceUpdatePayload>,
        command: PowerCommand,
    ) -> crate::Result<()> {
        if !self.peripheral.is_connected().await? {
            self.peripheral.connect().await?;
        }
        let characteristic = assert_power_characteristic(&self.peripheral).await?;
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
            self.send_power_status(&tx, power).await?;
            if stop {
                break;
            }
        }

        try_join!(
            self.peripheral.unsubscribe(&characteristic),
            self.peripheral.disconnect(),
        )
        .map_err(|_| crate::Error::Vrlh("Could not disconnect from device"))
        .map(|_| ())
    }

    async fn send_power_status(
        &self,
        tx: &Sender<DeviceUpdatePayload>,
        power: DevicePowerStatus,
    ) -> crate::Result<()> {
        tx.send(DeviceUpdatePayload::from_device(self, power))
            .await
            .map_err(Into::into)
    }
}
