use std::fmt::{Debug, Display};

use serde::Serialize;
use ts_rs::TS;

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
