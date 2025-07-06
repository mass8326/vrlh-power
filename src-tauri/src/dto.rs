use btleplug::platform::PeripheralId;
use serde::{ser::SerializeStruct, Serialize, Serializer};

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
    PoweredOn,
    PoweredOff,
    PowerPending,
    PowerInitiated,
    Unknown(Vec<u8>),
}

impl Serialize for DevicePowerStatus {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        let mut s = serializer.serialize_struct("StatusFile", 2)?;
        let code = match self {
            Self::Loading => "LOADING",
            Self::Error(_) => "ERROR",
            Self::PoweredOn => "POWERED_ON",
            Self::PoweredOff => "POWERED_OFF",
            Self::PowerPending => "POWER_PENDING",
            Self::PowerInitiated => "POWER_INITIATED",
            Self::Unknown(_) => "POWER_UNKNOWN",
        };
        let detail = match self {
            Self::Error(msg) => Some(msg.to_string()),
            Self::Unknown(bytes) => Some(format!("{bytes:?}").to_string()),
            _ => None,
        };
        s.serialize_field("code", code)?;
        s.serialize_field("detail", &detail)?;
        s.end()
    }
}

impl From<Vec<u8>> for DevicePowerStatus {
    fn from(value: Vec<u8>) -> Self {
        if value.len() != 1 {
            return Self::Unknown(value);
        };
        match value[0] {
            0 => Self::PoweredOff,
            1 => Self::PowerInitiated,
            9 => Self::PowerPending,
            11 => Self::PoweredOn,
            _ => Self::Unknown(value),
        }
    }
}
