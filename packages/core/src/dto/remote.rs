use std::fmt::{Debug, Write};

use serde::Serialize;
use ts_rs::TS;

#[derive(Clone, Serialize, TS)]
#[ts(export)]
pub enum DeviceRemoteStatus {
    Unavailable,
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
            Self::Unavailable => write!(f, "--"),
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
