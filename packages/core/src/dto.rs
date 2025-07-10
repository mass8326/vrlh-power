use std::fmt::{Debug, Display, Write};

use async_trait::async_trait;
use btleplug::platform::PeripheralId;
use serde::Serialize;
use tokio::sync::mpsc::{error::SendError, Sender};
use ts_rs::TS;

use crate::Device;

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

    pub fn from_device_local_status(device: &Device, status: DeviceLocalStatus) -> Self {
        Self {
            id: device.id(),
            addr: device.address(),
            name: device.name().to_string(),
            local: Some(status),
            remote: None,
        }
    }

    pub fn from_device_remote_status(device: &Device, status: DeviceRemoteStatus) -> Self {
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
pub trait SendDeviceStatus {
    async fn send_device_statuses(
        &self,
        device: &Device,
        local: DeviceLocalStatus,
        remote: DeviceRemoteStatus,
    ) -> Result<(), SendError<DeviceInfo>>;

    async fn send_device_local_status(
        &self,
        device: &Device,
        status: DeviceLocalStatus,
    ) -> Result<(), SendError<DeviceInfo>>;

    async fn send_device_remote_status(
        &self,
        device: &Device,
        status: DeviceRemoteStatus,
    ) -> Result<(), SendError<DeviceInfo>>;
}

#[async_trait]
impl SendDeviceStatus for Sender<DeviceInfo> {
    async fn send_device_statuses(
        &self,
        device: &Device,
        local: DeviceLocalStatus,
        remote: DeviceRemoteStatus,
    ) -> Result<(), SendError<DeviceInfo>> {
        self.send(DeviceInfo::from_device_statuses(device, local, remote))
            .await
    }

    async fn send_device_local_status(
        &self,
        device: &Device,
        status: DeviceLocalStatus,
    ) -> Result<(), SendError<DeviceInfo>> {
        self.send(DeviceInfo::from_device_local_status(device, status))
            .await
    }

    async fn send_device_remote_status(
        &self,
        device: &Device,
        status: DeviceRemoteStatus,
    ) -> Result<(), SendError<DeviceInfo>> {
        self.send(DeviceInfo::from_device_remote_status(device, status))
            .await
    }
}

pub enum DeviceCommand {
    Sleep,
    Activate,
    Standby,
}

impl From<DeviceCommand> for &[u8] {
    fn from(value: DeviceCommand) -> Self {
        match value {
            DeviceCommand::Sleep => &[0x00],
            DeviceCommand::Activate => &[0x01],
            DeviceCommand::Standby => &[0x02],
        }
    }
}

impl Display for DeviceCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            DeviceCommand::Activate => "ACTIVATE",
            DeviceCommand::Sleep => "SLEEP",
            DeviceCommand::Standby => "STANDBY",
        };
        write!(f, "{str}")
    }
}

#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub enum DeviceRemoteStatus {
    Stopped,
    Initiated,
    Standby,
    Acknowledged,
    Spinup,
    Active,
    Unknown(Vec<u8>),
}

impl Debug for DeviceRemoteStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stopped => write!(f, "00"),
            Self::Initiated => write!(f, "01"),
            Self::Standby => write!(f, "02"),
            Self::Acknowledged => write!(f, "08"),
            Self::Spinup => write!(f, "09"),
            Self::Active => write!(f, "0B"),
            Self::Unknown(bytes) => write!(
                f,
                "{}",
                bytes.iter().fold(String::new(), |mut result, byte| {
                    write!(result, "{byte:02X}").expect("Writing to string must not fail");
                    result
                })
            ),
        }
    }
}

impl From<Vec<u8>> for DeviceRemoteStatus {
    fn from(value: Vec<u8>) -> Self {
        if value.len() != 1 {
            return Self::Unknown(value);
        }
        match value.first().unwrap() {
            0x00 => Self::Stopped,
            0x01 => Self::Initiated,
            0x02 => Self::Standby,
            0x08 => Self::Acknowledged,
            0x09 => Self::Spinup,
            0x0B => Self::Active,
            _ => Self::Unknown(value),
        }
    }
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export)]
pub enum DeviceLocalStatus {
    Initializing,
    Disconnected,
    Connected,
    Ignored,
    FailConnection,
    FailVerify,
    Error(String),
}

impl Display for DeviceLocalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Self::Initializing => "INITIALIZING".into(),
            Self::Disconnected => "DISCONNECTED".into(),
            Self::Connected => "CONNECTED".into(),
            Self::Ignored => "IGNORED".into(),
            Self::FailConnection => "FAIL_CONNECTION".into(),
            Self::FailVerify => "FAIL_VERIFY".into(),
            Self::Error(str) => str.clone(),
        };
        write!(f, "{str}")
    }
}
