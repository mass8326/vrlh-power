use async_trait::async_trait;
use btleplug::platform::PeripheralId;
use tokio::sync::mpsc::{channel, Receiver};

use crate::dto::DeviceUpdatePayload;

use super::DeviceList;

#[async_trait]
pub trait PowerOn {
    async fn power_on(&self, id: PeripheralId) -> crate::Result<Receiver<DeviceUpdatePayload>>;
}

#[async_trait]
impl PowerOn for DeviceList {
    async fn power_on(&self, id: PeripheralId) -> crate::Result<Receiver<DeviceUpdatePayload>> {
        let (tx, rx) = channel(1);
        let device = self
            .map
            .clone()
            .lock()
            .await
            .get(&id)
            .ok_or(crate::Error::Vrlh("Device not found"))?
            .clone();
        tokio::spawn(async move { device.power_on(tx).await });
        Ok(rx)
    }
}
