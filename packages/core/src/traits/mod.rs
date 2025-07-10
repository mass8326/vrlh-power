use async_trait::async_trait;
use tokio::sync::mpsc::{error::SendError, Sender};

use crate::{Device, DeviceInfo, DeviceLocalStatus, DeviceRemoteStatus};

pub trait FromDeviceStatus<T> {
    fn from_device_status(device: &Device, status: T) -> Self;
}

impl FromDeviceStatus<DeviceLocalStatus> for DeviceInfo {
    fn from_device_status(device: &Device, status: DeviceLocalStatus) -> Self {
        Self {
            id: device.id(),
            addr: device.address(),
            name: device.name().to_string(),
            local: Some(status),
            remote: None,
        }
    }
}

impl FromDeviceStatus<DeviceRemoteStatus> for DeviceInfo {
    fn from_device_status(device: &Device, status: DeviceRemoteStatus) -> Self {
        Self {
            id: device.id(),
            addr: device.address(),
            name: device.name().to_string(),
            local: None,
            remote: Some(status),
        }
    }
}

#[async_trait]
pub trait SendDeviceStatus<T> {
    async fn send_device_status(
        &self,
        device: &Device,
        status: T,
    ) -> Result<(), SendError<DeviceInfo>>;
}

#[async_trait]
impl SendDeviceStatus<DeviceLocalStatus> for Sender<DeviceInfo> {
    async fn send_device_status(
        &self,
        device: &Device,
        status: DeviceLocalStatus,
    ) -> Result<(), SendError<DeviceInfo>> {
        let info = DeviceInfo::from_device_status(device, status);
        self.send(info).await
    }
}

#[async_trait]
impl SendDeviceStatus<DeviceRemoteStatus> for Sender<DeviceInfo> {
    async fn send_device_status(
        &self,
        device: &Device,
        status: DeviceRemoteStatus,
    ) -> Result<(), SendError<DeviceInfo>> {
        let info = DeviceInfo::from_device_status(device, status);
        self.send(info).await
    }
}
