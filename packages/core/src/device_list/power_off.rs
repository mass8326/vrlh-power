use async_trait::async_trait;
use btleplug::platform::PeripheralId;
use tokio::sync::mpsc::{channel, Receiver};

use crate::dto::DeviceUpdatePayload;

use super::DeviceList;

#[async_trait]
pub trait PowerOff {
    async fn power_off(&self, id: PeripheralId) -> crate::Result<Receiver<DeviceUpdatePayload>>;
}

#[async_trait]
impl PowerOff for DeviceList {
    async fn power_off(&self, id: PeripheralId) -> crate::Result<Receiver<DeviceUpdatePayload>> {
        let (tx, rx) = channel(1);
        let device = self
            .map
            .clone()
            .lock()
            .await
            .get(&id)
            .ok_or(crate::Error::Vrlh("Device not found"))?
            .clone();
        tokio::spawn(async move { device.power_off(tx).await });
        Ok(rx)
    }
}
