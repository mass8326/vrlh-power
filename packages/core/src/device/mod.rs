use std::sync::{Arc, Mutex};

use btleplug::{
    api::{Characteristic, Peripheral as _, WriteType},
    platform::{Peripheral, PeripheralId},
};
use futures::{StreamExt, TryFutureExt};
use tokio::sync::mpsc::Sender;

use crate::{
    constants::{LHV2_GATT_POWER_CHARACTERISTIC, LHV2_GATT_POWER_SERVICE},
    DeviceCommand, DeviceInfo, DeviceLocalStatus, DeviceRemoteStatus, SendDeviceStatus,
};

#[derive(Clone, Debug)]
pub struct Device {
    peripheral: Peripheral,
    name: String,
    characteristic: Arc<Mutex<Option<Characteristic>>>,
    local: Arc<Mutex<DeviceLocalStatus>>,
    remote: Arc<Mutex<DeviceRemoteStatus>>,
}

impl Device {
    pub fn new(peripheral: Peripheral, name: String) -> Self {
        Self {
            peripheral,
            name,
            characteristic: Arc::new(Mutex::new(None)),
            local: Arc::new(Mutex::new(DeviceLocalStatus::Initializing)),
            remote: Arc::new(Mutex::new(DeviceRemoteStatus::Unavailable)),
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
        tx: Sender<DeviceRemoteStatus>,
        command: &DeviceCommand,
    ) -> crate::Result<()> {
        self.ensure_connected().await?;
        let characteristic = self.get_power_characteristic().await?;
        self.peripheral.subscribe(&characteristic).await?;
        let mut events = self.peripheral.notifications().await?;

        self.peripheral
            .write(&characteristic, command.into(), WriteType::WithResponse)
            .await?;
        while let Some(event) = events.next().await {
            let remote = DeviceRemoteStatus::from(event.value);
            let stop = matches!(
                remote,
                DeviceRemoteStatus::Active
                    | DeviceRemoteStatus::Standby
                    | DeviceRemoteStatus::Stopped
            );
            tx.send(remote).await?;
            if stop {
                break;
            }
        }

        self.peripheral
            .unsubscribe(&characteristic)
            .await
            .map_err(|_| crate::Error::Vrlh("Could not unsubscribe from device"))?;
        self.disconnect().await?;
        Ok(())
    }

    pub async fn ensure_connected(&self) -> crate::Result<()> {
        if !self.peripheral.is_connected().await? {
            self.peripheral.connect().await?;
        }
        Ok(())
    }

    pub async fn disconnect(&self) -> crate::Result<()> {
        self.peripheral.disconnect().await?;
        Ok(())
    }

    pub fn get_last_statuses(&self) -> (DeviceLocalStatus, DeviceRemoteStatus) {
        (
            self.local
                .lock()
                .expect("Device local status mutex should not be poisoned")
                .clone(),
            self.remote
                .lock()
                .expect("Device remote status mutex should not be poisoned")
                .clone(),
        )
    }

    pub async fn fetch_remote_status(&self, tx: Sender<DeviceInfo>) -> crate::Result<()> {
        let _ = tx
            .send_device_local_status(self, DeviceLocalStatus::Initializing)
            .await;

        self.ensure_connected()
            .and_then(async |()| {
                let _ = tx
                    .send_device_local_status(self, DeviceLocalStatus::Connected)
                    .await;
                Ok(())
            })
            .or_else(async |error| {
                let _ = tx
                    .send_device_local_status(self, DeviceLocalStatus::FailConnection)
                    .await;
                Err(error)
            })
            .await?;

        let result = self
            .get_power_characteristic()
            .and_then(async |char| self.peripheral.read(&char).await.map_err(Into::into))
            .and_then(async |bytes| {
                let _ = tx
                    .send_device_remote_status(self, DeviceRemoteStatus::from(bytes))
                    .await;
                Ok(())
            })
            .await;

        let disconnect_status = match self.disconnect().await {
            Ok(()) => DeviceLocalStatus::Disconnected,
            Err(_) => DeviceLocalStatus::FailConnection,
        };
        let _ = tx.send_device_local_status(self, disconnect_status).await;

        result
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
