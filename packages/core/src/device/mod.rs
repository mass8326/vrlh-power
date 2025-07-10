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
    local: Arc<Mutex<DeviceLocalStatus>>,
    remote: Arc<Mutex<DeviceRemoteStatus>>,
}

impl Device {
    pub fn new(peripheral: Peripheral, name: String) -> Self {
        Self {
            peripheral,
            name,
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
        tx: Sender<DeviceInfo>,
        command: DeviceCommand,
    ) -> crate::Result<()> {
        self.ensure_connected(tx.clone()).await?;
        let tx_clone = tx.clone();
        let result = self
            .get_power_characteristic()
            .and_then(async |char| {
                let maybe_events = self
                    .peripheral
                    .subscribe(&char)
                    .and_then(|()| self.peripheral.notifications())
                    .await;
                self.peripheral
                    .write(&char, command.into(), WriteType::WithResponse)
                    .await?;
                if let Ok(mut events) = maybe_events {
                    while let Some(event) = events.next().await {
                        let remote = DeviceRemoteStatus::from(event.value);
                        let stop = matches!(
                            remote,
                            DeviceRemoteStatus::Active
                                | DeviceRemoteStatus::Standby
                                | DeviceRemoteStatus::Stopped
                        );
                        let _ = tx_clone.send_device_remote_status(self, remote).await;
                        if stop {
                            break;
                        }
                    }
                } else {
                    let _ = tx_clone
                        .send_device_local_status(self, DeviceLocalStatus::FailVerify)
                        .await;
                }
                Ok::<(), crate::Error>(())
            })
            .await;
        let disconnect_status = match self.disconnect().await {
            Ok(()) => DeviceLocalStatus::Disconnected,
            Err(_) => DeviceLocalStatus::FailConnection,
        };
        let _ = tx.send_device_local_status(self, disconnect_status).await;
        result
    }

    pub async fn ensure_connected(&self, tx: Sender<DeviceInfo>) -> crate::Result<()> {
        if self.peripheral.is_connected().await? {
            return Ok(());
        }
        self.peripheral
            .connect()
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
        self.ensure_connected(tx.clone()).await?;
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

    /// The characteristic must be used during the same connection session during which it was retrieved
    pub async fn get_power_characteristic(&self) -> crate::Result<Characteristic> {
        self.peripheral.discover_services().await?;
        let service = self
            .peripheral
            .services()
            .into_iter()
            .find(|service| service.uuid == LHV2_GATT_POWER_SERVICE)
            .ok_or(crate::Error::Vrlh("Could not verify power service!"))?;
        service
            .characteristics
            .into_iter()
            .find(|char| char.uuid == LHV2_GATT_POWER_CHARACTERISTIC)
            .ok_or(crate::Error::Vrlh("Could not verify power charateristic!"))
    }
}
