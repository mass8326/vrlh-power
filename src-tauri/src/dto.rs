use btleplug::platform::PeripheralId;
use serde::{Serialize, Serializer};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceUpdatePayload<'a> {
    pub id: &'a PeripheralId,
    pub name: Option<&'a str>,
    pub power: &'a DevicePowerStatus,
}

#[derive(Debug, Clone)]
pub enum DevicePowerStatus {
    Loading,
    Error(String),
    PowerOn,
    PowerOff,
    PowerPending,
    PowerInitiated,
    Unknown(Vec<u8>),
}

impl Serialize for DevicePowerStatus {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let str = match self {
            Self::Loading => "Loading status...",
            Self::Error(msg) => &msg,
            Self::PowerOn => "Power on",
            Self::PowerOff => "Power off",
            Self::PowerPending => "Power pending...",
            Self::PowerInitiated => "Power initiated...",
            Self::Unknown(bytes) => &format!("Unknown status ({bytes:?})"),
        };
        serializer.serialize_str(str)
    }
}

impl From<Vec<u8>> for DevicePowerStatus {
    fn from(value: Vec<u8>) -> Self {
        if value.len() != 1 {
            return Self::Unknown(value);
        };
        match value[0] {
            0 => Self::PowerOff,
            1 => Self::PowerInitiated,
            9 => Self::PowerPending,
            11 => Self::PowerOn,
            _ => Self::Unknown(value),
        }
    }
}
