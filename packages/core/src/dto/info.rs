use std::fmt::Debug;

use btleplug::platform::PeripheralId;
use serde::Serialize;
use ts_rs::TS;

use crate::{Device, DeviceLocalStatus, DeviceRemoteStatus};

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
pub struct DeviceInfo {
    /// Serializes differently per platform
    #[ts(type = "unknown")]
    pub id: PeripheralId,
    pub addr: String,
    pub name: String,
    pub local: Option<DeviceLocalStatus>,
    pub remote: Option<DeviceRemoteStatus>,
}

impl DeviceInfo {
    pub fn from_device_statuses(
        device: &Device,
        local: DeviceLocalStatus,
        remote: DeviceRemoteStatus,
    ) -> Self {
        Self {
            id: device.id(),
            addr: device.address(),
            name: device.name().to_string(),
            local: Some(local),
            remote: Some(remote),
        }
    }
}
