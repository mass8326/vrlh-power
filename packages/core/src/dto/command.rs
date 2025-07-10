use std::fmt::Display;

#[derive(Clone, Debug)]
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
